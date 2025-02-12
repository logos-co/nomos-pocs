/// Constraint No-op Proof
use ledger_proof_statements::covenant::SpendingCovenantPublic;
use risc0_zkvm::guest::env;

fn main() {
    let public: SpendingCovenantPublic = env::read();
    env::commit(&public);
}
