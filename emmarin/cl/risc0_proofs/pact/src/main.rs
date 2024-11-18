/// Input Proof
use cl::merkle;
use ledger_proof_statements::pact::{PactPrivate, PactPublic};
use risc0_zkvm::guest::env;

fn main() {
    let PactPrivate {
        pact,
        input_cm_paths,
        cm_root,
    } = env::read();

    assert_eq!(pact.tx.inputs.len(), input_cm_paths.len());
    for (input, cm_path) in pact.tx.inputs.iter().zip(input_cm_paths) {
        let note_cm = input.note_commitment(&pact.from);
        let cm_leaf = merkle::leaf(note_cm.as_bytes());
        assert_eq!(cm_root, merkle::path_root(cm_leaf, &cm_path));
    }

    for output in pact.tx.outputs.iter() {
        assert!(output.note.value > 0);
    }

    assert!(pact.tx.balance().is_zero());

    env::commit(&PactPublic {
        pact: pact.commit(),
        cm_root,
    });
}
