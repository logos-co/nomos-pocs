use cl::{
    merkle,
    nullifier::{Nullifier, NullifierNonce, NullifierSecret},
    partial_tx::{MAX_INPUTS, MAX_OUTPUTS},
    PtxRoot,
};

use common::*;
use goas_proof_statements::zone_funds::Spend;
use proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

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
    assert_ne!(deposit.note.death_constraint, ZONE_FUNDS_VK);

    // 3) Check the ptx is balanced. This is not a requirement for standard ptxs, but we need it
    //    in deposits (at least in a first version) to ensure fund tracking
    assert_eq!(deposit.note.balance.unit, *ZONE_UNIT);
    assert_eq!(zone_funds_in.note.balance.unit, *ZONE_UNIT);
    assert_eq!(zone_funds_out.note.balance.unit, *ZONE_UNIT);

    let in_sum = deposit.note.balance.value + zone_funds_in.note.balance.value;

    let out_sum = zone_note_out.note.balance.value;

    assert_eq!(out_sum, in_sum, "deposit ptx is unbalanced");

    // 4) Check the zone fund notes are correctly created
    assert_eq!(zone_funds_in.note.death_constraint, ZONE_FUNDS_VK);
    assert_eq!(zone_funds_out.note.death_constraint, ZONE_FUNDS_VK);
    assert_eq!(zone_funds_in.nf_sk, NullifierSecret::from_bytes([0; 16])); // there is no secret in the zone funds
    assert_eq!(zone_funds_out.nf_pk, zone_funds_in.nf_sk.commit()); // the sk is the same
                                                                    // nonce is correctly evolved
    let mut evolved_nonce = [0; 16];
    evolved_nonce[..16].copy_from_slice(&Sha256::digest(&zone_funds_in.nonce.as_bytes())[..16]);
    assert_eq!(
        zone_funds_out.nonce,
        NullifierNonce::from_bytes(evolved_nonce)
    );

    // 5) Check zone state notes are correctly created
    assert_eq!(
        zone_note_in.note.death_constraint,
        zone_note_out.note.death_constraint
    );
    assert_eq!(zone_note_in.nf_sk, NullifierSecret::from_bytes([0; 16])); //// there is no secret in the zone state
    assert_eq!(zone_note_out.nf_pk, zone_note_in.nf_sk.commit()); // the sk is the same
    assert_eq!(
        zone_note_in.note.balance.unit,
        zone_note_out.note.balance.unit
    );
    assert_eq!(
        zone_note_in.note.balance.value,
        zone_note_out.note.balance.value
    );
    // nonce is correctly evolved
    let mut evolved_nonce = [0; 16];
    evolved_nonce[..16].copy_from_slice(&Sha256::digest(&zone_note_in.nonce.as_bytes())[..16]);
    assert_eq!(
        zone_note_out.nonce,
        NullifierNonce::from_bytes(evolved_nonce)
    );

    let nullifier = Nullifier::new(zone_note_in.nf_sk, zone_note_in.nonce);
    assert_eq!(nullifier, pub_inputs.nf);

    // 6) We're now ready to do the deposit!

    let amount = deposit.note.balance.value as u32;
    let to = AccountId::from_be_bytes(<[u8; 4]>::try_from(&deposit.note.state[0..4]).unwrap());

    let to_balance = state.balances.entry(to).or_insert(0);
    *to_balance = to_balance
        .checked_add(amount)
        .expect("overflow when depositing");

    state
}

fn main() {
    let public_inputs: DeathConstraintPublic = env::read();
    let inputs: Vec<Input> = env::read();
    let mut state: StateWitness = env::read();

    for input in inputs {
        match input {
            Input::Withdraw(input) => state = withdraw(state, input),
            Input::Deposit(input) => state = deposit(state, input, public_inputs),
        }
    }

    env::commit(&state.commit());
}
