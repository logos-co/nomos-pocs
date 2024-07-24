/// Death Constraint No-op Proof
use proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let public: DeathConstraintPublic = env::read();
    env::commit(&public);
}
