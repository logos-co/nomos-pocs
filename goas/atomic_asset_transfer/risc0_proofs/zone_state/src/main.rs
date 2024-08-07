use cl::{
    merkle,
    nullifier::{Nullifier, NullifierSecret},
    partial_tx::{MAX_INPUTS, MAX_OUTPUTS},
    PtxRoot,
};

use common::*;
use goas_proof_statements::zone_state::ZoneStatePrivate;
use ledger_proof_statements::{
    death_constraint::DeathConstraintPublic,
};
use risc0_zkvm::guest::env;

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
    assert_eq!(zone_funds_out.nonce, zone_funds_in.evolved_nonce());

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
    assert_eq!(zone_note_out.nonce, zone_note_in.evolved_nonce());
    let nullifier = Nullifier::new(zone_note_in.nf_sk, zone_note_in.nonce);
    assert_eq!(nullifier, pub_inputs.nf);

    // 6) We're now ready to do the deposit!
    let amount = deposit.note.value;
    let to = AccountId::from_be_bytes(<[u8; 4]>::try_from(&deposit.note.state[0..4]).unwrap());

    let to_balance = state.balances.entry(to).or_insert(0);
    *to_balance = to_balance
        .checked_add(amount)
        .expect("overflow when depositing");

    state
}

fn validate_zone_transition(
    in_note: &cl::PartialTxInputWitness,
    out_note: &cl::PartialTxOutputWitness,
    in_state_cm: &StateCommitment,
    out_state_cm: &StateCommitment,
) {
    // Ensure input/output notes are committing to the expected states.
    assert_eq!(in_note.input.note.state, in_state_cm.0);
    assert_eq!(out_note.output.note.state, out_state_cm.0);

    // death constraint is propagated
    assert_eq!(
        in_note.input.note.death_constraint,
        out_note.output.note.death_constraint
    );
    // nullifier secret is propagated
    assert_eq!(in_note.input.nf_sk.commit(), out_note.output.nf_pk);
    // balance blinding is propagated
    assert_eq!(
        in_note.input.balance_blinding,
        out_note.output.balance_blinding
    );
    // unit is propagated
    assert_eq!(in_note.input.note.unit, out_note.output.note.unit);
    // the nonce is correctly evolved
    assert_eq!(in_note.input.evolved_nonce(), out_note.output.nonce);
}

fn main() {
    let ZoneStatePrivate {
        mut state,
        inputs,
        zone_in,
        zone_out,
    } = env::read();

    let pub_inputs = DeathConstraintPublic {
        ptx_root: PtxRoot(cl::merkle::node(
            zone_in.input_root(),
            zone_out.output_root(),
        )),
        nf: zone_in.input.nullifier(),
    };

    let in_state_cm = state.commit();

    for input in inputs {
        state = match input {
            Input::Withdraw(input) => state.withdraw(input),
            Input::Deposit(input) => deposit(state, input, pub_inputs),
        }
    }

    let out_state_cm = state.commit();

    validate_zone_transition(&zone_in, &zone_out, &in_state_cm, &out_state_cm);

    env::commit(&pub_inputs);
}
