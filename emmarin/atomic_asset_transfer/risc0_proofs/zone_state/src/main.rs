use cl::{note::NoteWitness, output::OutputWitness, PtxRoot};

use common::*;
use goas_proof_statements::zone_state::ZoneStatePrivate;
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

fn validate_zone_transition(
    in_note: cl::PartialTxInputWitness,
    out_note: cl::PartialTxOutputWitness,
    in_state_cm: StateCommitment,
    out_state: StateWitness,
    in_ledger_cm: Ledger,
    out_ledger: LedgerWitness,
) {
    let metadata = out_state.zone_metadata;
    let out_state_cm = out_state.commit().0;
    let out_ledger_cm = out_ledger.commit().0;
    // Ensure input/output notes are committing to the expected states.
    assert_eq!(in_note.input.note.state, in_state_cm.0);
    assert_eq!(in_note.input.note.ledger.in_ledger_cm);
    assert_eq!(out_note.output.note.state, out_state_cm);
    assert_eq!(out_note.output.note.ledger, out_ledger_cm);

    // ensure units match metadata
    assert_eq!(in_note.input.note.unit, metadata.unit);
    assert_eq!(out_note.output.note.unit, metadata.unit);

    // ensure constraints match metadata
    assert_eq!(in_note.input.note.constraint, metadata.zone_constraint);
    assert_eq!(out_note.output.note.constraint, metadata.zone_constraint);

    // nullifier secret is propagated
    assert_eq!(in_note.input.nf_sk.commit(), out_note.output.nf_pk);

    // the nonce is correctly evolved
    assert_eq!(
        in_note.input.evolved_nonce(b"STATE_NONCE"),
        out_note.output.note.nonce
    );
}

fn main() {
    let ZoneStatePrivate {
        mut state,
        mut ledger,
        zone_in,
        txs,
        zone_out,
    } = env::read();

    let ptx_root = zone_in.ptx_root();
    assert_eq!(ptx_root, zone_out.ptx_root());

    let pub_inputs = ConstraintPublic {
        ptx_root,
        nf: zone_in.input.nullifier(),
    };

    let in_state_cm = state.commit();
    let in_ledger_cm = ledger.commit();

    for tx in txs {
        match tx {
            Transfer(t) => {
                env::verify(emmarin_risc0_proofs::TRANSFER_ID, serde::to_vec(t).unwrap()).unwrap();
                assert_eq!(t.cm_root, in_ledger_cm.cm_root);
                ledger.commitments.push(t.cm);
                ledger.nullifiers.push(t.nf);
            }
            Withdraw(w) => {
                env::verify(emmarin_risc0_proofs::WITHDRAW_ID, serde::to_vec(w).unwrap()).unwrap();
                assert_eq!(w.cm_root, in_ledger_cm.cm_root);
                ledger.nullifier.push(w.nf);
            }
        }
    }

    validate_zone_transition(zone_in, zone_out, in_state_cm, state, in_ledger_cm, ledger);

    env::commit(&pub_inputs);
}
