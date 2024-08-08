use cl::{
    note::NoteWitness,
    nullifier::NullifierNonce,
    output::OutputWitness,
    partial_tx::{PartialTxInputWitness, PartialTxOutputWitness},
    PtxRoot,
};

use common::*;
use goas_proof_statements::zone_state::ZoneStatePrivate;
use ledger_proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;

fn withdraw(
    state: StateWitness,
    output_root: [u8; 32],
    withdrawal_req: Withdraw,
    withdrawal: PartialTxOutputWitness,
) -> StateWitness {
    // 1) check the correct amount of funds is being spent
    assert_eq!(withdrawal.output.note.value, withdrawal_req.amount);
    assert_eq!(withdrawal.output.note.unit, *ZONE_CL_FUNDS_UNIT);
    // 2) check the correct recipient is being paid
    assert_eq!(withdrawal.output.nf_pk, withdrawal_req.to);

    assert_eq!(output_root, withdrawal.output_root());

    state.withdraw(withdrawal_req)
}

fn deposit(
    state: StateWitness,
    input_root: [u8; 32],
    deposit_req: Deposit,
    deposit: PartialTxInputWitness,
) -> StateWitness {
    assert_eq!(deposit.input_root(), input_root);

    // 1) Check the deposit note is not already under control of the zone
    assert_ne!(
        deposit.input.note.death_constraint,
        state.zone_metadata.funds_vk
    );

    // 2) Check the deposit note is for the correct amount
    assert_eq!(deposit.input.note.unit, *ZONE_CL_FUNDS_UNIT);
    assert_eq!(deposit.input.note.value, deposit_req.amount);

    // 3) Check the deposit note is for the correct recipient
    assert_eq!(
        AccountId::from_le_bytes(<[u8; 4]>::try_from(&deposit.input.note.state[..4]).unwrap()),
        deposit_req.to
    );

    state.deposit(deposit_req)
}

fn validate_zone_transition(
    in_note: cl::PartialTxInputWitness,
    out_note: cl::PartialTxOutputWitness,
    out_funds: cl::PartialTxOutputWitness,
    in_state_cm: StateCommitment,
    out_state: StateWitness,
) {
    let metadata = out_state.zone_metadata;
    let out_state_cm = out_state.commit().0;
    // Ensure input/output notes are committing to the expected states.
    assert_eq!(in_note.input.note.state, in_state_cm.0);
    assert_eq!(out_note.output.note.state, out_state_cm);

    // zone metadata is propagated
    assert_eq!(out_state.zone_metadata.id(), metadata.id());

    // ensure units match metadata
    assert_eq!(in_note.input.note.unit, metadata.unit);
    assert_eq!(out_note.output.note.unit, metadata.unit);

    // ensure constraints match metadata
    assert_eq!(in_note.input.note.death_constraint, metadata.zone_vk);
    assert_eq!(out_note.output.note.death_constraint, metadata.zone_vk);

    // nullifier secret is propagated
    assert_eq!(in_note.input.nf_sk.commit(), out_note.output.nf_pk);

    // balance blinding is propagated
    assert_eq!(
        in_note.input.balance_blinding,
        out_note.output.balance_blinding
    );

    // the nonce is correctly evolved
    assert_eq!(in_note.input.evolved_nonce(), out_note.output.nonce);

    // funds are still under control of the zone
    let expected_note_witness = NoteWitness::new(
        out_state.total_balance(),
        *ZONE_CL_FUNDS_UNIT,
        metadata.funds_vk,
        metadata.id(),
    );
    assert_eq!(
        out_funds.output,
        OutputWitness::public(
            expected_note_witness,
            NullifierNonce::from_bytes(out_state.nonce)
        )
    );
    // funds belong to the same partial tx
    assert_eq!(out_funds.output_root(), out_note.output_root());
}

fn main() {
    let ZoneStatePrivate {
        mut state,
        inputs,
        zone_in,
        zone_out,
        funds_out,
        mut withdrawals,
        mut deposits,
    } = env::read();

    let input_root = zone_in.input_root();
    let output_root = zone_out.output_root();

    let pub_inputs = DeathConstraintPublic {
        ptx_root: PtxRoot(cl::merkle::node(input_root, output_root)),
        nf: zone_in.input.nullifier(),
    };

    let in_state_cm = state.commit();

    for input in inputs {
        state = match input {
            Input::Withdraw(w) => withdraw(state, output_root, w, withdrawals.pop_front().unwrap()),
            Input::Deposit(d) => deposit(state, input_root, d, deposits.pop_front().unwrap()),
        }
    }

    let state = state.evolve_nonce();
    validate_zone_transition(zone_in, zone_out, funds_out, in_state_cm, state);

    env::commit(&pub_inputs);
}
