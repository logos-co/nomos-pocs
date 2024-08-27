/// Constraint No-op Proof
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let public: ConstraintPublic = env::read();
    env::commit(&public);
}
