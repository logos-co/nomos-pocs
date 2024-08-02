use cl::{
    input::InputWitness,
    merkle,
    nullifier::{Nullifier, NullifierSecret},
    partial_tx::{MAX_INPUTS, MAX_OUTPUTS},
    PtxRoot,
};

use common::*;
use goas_proof_statements::zone_funds::Spend;
use proof_statements::{
    death_constraint::DeathConstraintPublic,
    ptx::{PartialTxInputPrivate, PartialTxOutputPrivate},
};
use risc0_zkvm::guest::env;

fn withdraw(mut state: StateWitness, withdraw: Withdraw) -> StateWitness {
    state.included_txs.push(Input::Withdraw(withdraw));

    let Withdraw {
        from,
        amount,
        to,
        nf,
    } = withdraw;

    let from_balance = state.balances.entry(from).or_insert(0);
    *from_balance = from
        .checked_sub(amount)
        .expect("insufficient funds in account");
    let spend_auth = Spend {
        amount: amount.into(),
        to,
        nf,
    };

    state.output_events.push(Event::Spend(spend_auth));
    state
}

fn deposit(
    mut state: StateWitness,
    deposit: Deposit,
    pub_inputs: DeathConstraintPublic,
) -> StateWitness {
    state.included_txs.push(Input::Deposit(deposit.clone()));

    let Deposit {
        deposit,
        zone_note_in,
        zone_note_out,
        zone_funds_in,
        zone_funds_out,
    } = deposit;

    let funds_vk = state.zone_metadata.funds_vk;

    // 1) Check there are no more input/output notes than expected
    let inputs = [
        deposit.commit().to_bytes().to_vec(),
        zone_note_in.commit().to_bytes().to_vec(),
        zone_funds_in.commit().to_bytes().to_vec(),
    ];

    let inputs_root = merkle::root(merkle::padded_leaves::<MAX_INPUTS>(&inputs));

    let outputs = [
        zone_note_out.commit().to_bytes().to_vec(),
        zone_funds_out.commit().to_bytes().to_vec(),
    ];

    let outputs_root = merkle::root(merkle::padded_leaves::<MAX_OUTPUTS>(&outputs));

    let ptx_root = PtxRoot(merkle::node(inputs_root, outputs_root));
    assert_eq!(ptx_root, pub_inputs.ptx_root);

    // 2) Check the deposit note is not already under control of the zone
    assert_ne!(deposit.note.death_constraint, funds_vk);

    // 3) Check the ptx is balanced. This is not a requirement for standard ptxs, but we need it
    //    in deposits (at least in a first version) to ensure fund tracking
    assert_eq!(deposit.note.unit, *ZONE_CL_FUNDS_UNIT);
    assert_eq!(zone_funds_in.note.unit, *ZONE_CL_FUNDS_UNIT);
    assert_eq!(zone_funds_out.note.unit, *ZONE_CL_FUNDS_UNIT);

    let in_sum = deposit.note.value + zone_funds_in.note.value;

    let out_sum = zone_note_out.note.value;

    assert_eq!(out_sum, in_sum, "deposit ptx is unbalanced");

    // 4) Check the zone fund notes are correctly created
    assert_eq!(zone_funds_in.note.death_constraint, funds_vk);
    assert_eq!(zone_funds_out.note.death_constraint, funds_vk);
    assert_eq!(zone_funds_in.note.state, state.zone_metadata.id());
    assert_eq!(zone_funds_out.note.state, state.zone_metadata.id());
    assert_eq!(zone_funds_in.nf_sk, NullifierSecret::from_bytes([0; 16])); // there is no secret in the zone funds
    assert_eq!(zone_funds_out.nf_pk, zone_funds_in.nf_sk.commit()); // the sk is the same
                                                                    // nonce is correctly evolved
    assert_eq!(
        zone_funds_out.nonce,
        zone_funds_in
            .nonce
            .evolve(&NullifierSecret::from_bytes([0; 16]))
    );

    // 5) Check zone state notes are correctly created
    assert_eq!(
        zone_note_in.note.death_constraint,
        zone_note_out.note.death_constraint
    );
    assert_eq!(zone_note_in.nf_sk, NullifierSecret::from_bytes([0; 16])); //// there is no secret in the zone state
    assert_eq!(zone_note_out.nf_pk, zone_note_in.nf_sk.commit()); // the sk is the same
    assert_eq!(zone_note_in.note.unit, zone_note_out.note.unit);
    assert_eq!(zone_note_in.note.value, zone_note_out.note.value);
    // nonce is correctly evolved
    assert_eq!(
        zone_note_out.nonce,
        zone_note_in
            .nonce
            .evolve(&NullifierSecret::from_bytes([0; 16]))
    );
    let nullifier = Nullifier::new(zone_note_in.nf_sk, zone_note_in.nonce);
    assert_eq!(nullifier, pub_inputs.nf);

    // 6) We're now ready to do the deposit!
    let amount = deposit.note.value as u32;
    let to = AccountId::from_be_bytes(<[u8; 4]>::try_from(&deposit.note.state[0..4]).unwrap());

    let to_balance = state.balances.entry(to).or_insert(0);
    *to_balance = to_balance
        .checked_add(amount)
        .expect("overflow when depositing");

    state
}

fn validate_zone_input(
    input: &PartialTxInputPrivate,
    state: &StateWitness,
) -> (PtxRoot, Nullifier) {
    let ptx_root = input.ptx_root();
    let nf = Nullifier::new(input.input.nf_sk, input.input.nonce);

    assert_eq!(input.input.note.state, <[u8; 32]>::from(state.commit()));
    // should not be possible to create one but let's put this check here just in case
    debug_assert_eq!(
        input.input.note.death_constraint,
        state.zone_metadata.zone_vk
    );

    (ptx_root, nf)
}

fn validate_zone_output(
    ptx: PtxRoot,
    input: InputWitness,
    output: PartialTxOutputPrivate,
    state: &StateWitness,
) {
    assert_eq!(ptx, output.ptx_root()); // the ptx root is the same as in the input
    let output = output.output;
    assert_eq!(output.note.state, <[u8; 32]>::from(state.commit())); // the state in the output is as calculated by this function
    assert_eq!(output.note.death_constraint, state.zone_metadata.zone_vk); // the death constraint is the correct one
    assert_eq!(output.nf_pk, NullifierSecret::from_bytes([0; 16]).commit()); // the nullifier secret is public
    assert_eq!(output.balance_blinding, input.balance_blinding); // the balance blinding is the same as in the input
    assert_eq!(output.note.unit, state.zone_metadata.unit); // the balance unit is the same as in the input

    // the nonce is correctly evolved
    assert_eq!(
        output.nonce,
        input.nonce.evolve(&NullifierSecret::from_bytes([0; 16]))
    );
}

fn main() {
    let zone_in: PartialTxInputPrivate = env::read();
    let mut state: StateWitness = env::read();
    let zone_out: PartialTxOutputPrivate = env::read();

    let (ptx_root, nf) = validate_zone_input(&zone_in, &state);

    let pub_inputs = DeathConstraintPublic { ptx_root, nf };

    let inputs: Vec<Input> = env::read();

    for input in inputs {
        match input {
            Input::Withdraw(input) => state = withdraw(state, input),
            Input::Deposit(input) => state = deposit(state, input, pub_inputs),
        }
    }

    validate_zone_output(ptx_root, zone_in.input, zone_out, &state);
    env::commit(&pub_inputs);
}
