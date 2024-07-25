/// Proof of Leadership
use cl::merkle;
use curve25519_dalek::Scalar;
use proof_statements::proof_of_leadership::{LeaderPrivate, LeaderPublic};
use risc0_zkvm::guest::env;


fn main() {
    let public_inputs: LeaderPublic = env::read();

    let LeaderPrivate {
	input,
	input_cm_path,
    } = env::read();

    // Lottery checks
    assert!(public_inputs.check_winning(&input));


    // Ensure note is valid
    let note_cm = input.note_commitment();
    let note_cm_leaf = merkle::leaf(note_cm.as_bytes());
    let note_cm_root = merkle::path_root(note_cm_leaf, &input_cm_path);
    assert_eq!(note_cm_root, public_inputs.cm_root);


    // Public input constraints
    assert_eq!(input.nullifier(), public_inputs.nullifier);

    let evolved_output = input.evolve_output(cl::BalanceWitness::new(Scalar::ZERO));
    assert_eq!(evolved_output.commit_note(), public_inputs.updated_commitment);
	
    env::commit(&public_inputs);
}
