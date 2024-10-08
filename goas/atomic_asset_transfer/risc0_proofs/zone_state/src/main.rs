use cl::{
    note::NoteWitness, output::OutputWitness,
    PtxRoot,
};

use common::*;
use goas_proof_statements::zone_state::ZoneStatePrivate;
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

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

    // ensure units match metadata
    assert_eq!(in_note.input.note.unit, metadata.unit);
    assert_eq!(out_note.output.note.unit, metadata.unit);

    // ensure constraints match metadata
    assert_eq!(in_note.input.note.constraint, metadata.zone_constraint);
    assert_eq!(out_note.output.note.constraint, metadata.zone_constraint);

    // nullifier secret is propagated
    assert_eq!(in_note.input.nf_sk.commit(), out_note.output.nf_pk);

    // the nonce is correctly evolved
    assert_eq!(in_note.input.evolved_nonce(b"STATE_NONCE"), out_note.output.note.nonce);

    // funds are still under control of the zone
    let expected_note_witness = NoteWitness {
        value: out_state.total_balance(),
        unit: *ZONE_CL_FUNDS_UNIT,
        constraint: metadata.funds_constraint,
        state: metadata.id(),
        nonce: in_note.input.evolved_nonce(b"FUND_NONCE")
    };
    assert_eq!(
        out_funds.output,
        OutputWitness::public(expected_note_witness)
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
    } = env::read();

    let input_root = zone_in.input_root();
    let output_root = zone_out.output_root();

    let pub_inputs = ConstraintPublic {
        ptx_root: PtxRoot(cl::merkle::node(input_root, output_root)),
        nf: zone_in.input.nullifier(),
    };

    let in_state_cm = state.commit();

    for (signed_bound_tx, ptx_input_witness) in inputs {
        // verify the signature
        let bound_tx = signed_bound_tx.verify_and_unwrap();

        // ensure the note this tx is bound to is present in the ptx
        assert_eq!(bound_tx.bind, ptx_input_witness.input.note_commitment());
        assert_eq!(ptx_input_witness.input_root(), input_root);

        // apply the ptx
        state = state.apply(bound_tx.tx).0
    }

    validate_zone_transition(zone_in, zone_out, funds_out, in_state_cm, state);

    env::commit(&pub_inputs);
}
