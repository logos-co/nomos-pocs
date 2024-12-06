/// Input Proof
use ledger_proof_statements::ptx::{PtxPrivate, PtxPublic};
use risc0_zkvm::guest::env;

fn main() {
    let PtxPrivate {
        ptx,
        input_cm_proofs,
        cm_mmr,
    } = env::read();

    assert_eq!(ptx.inputs.len(), input_cm_proofs.len());
    for (input, cm_mmr_proof) in ptx.inputs.iter().zip(input_cm_proofs) {
        let note_cm = input.note_commitment();
        assert!(cm_mmr.verify_proof(&note_cm.0, &cm_mmr_proof));
    }

    for output in ptx.outputs.iter() {
        assert!(output.note.value > 0);
    }

    env::commit(&PtxPublic {
        ptx: ptx.commit(),
        cm_mmr,
    });
}
