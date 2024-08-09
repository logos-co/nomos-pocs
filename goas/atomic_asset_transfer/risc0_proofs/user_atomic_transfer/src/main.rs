use goas_proof_statements::user_note::UserAtomicTransfer;
use ledger_proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let transfer: UserAtomicTransfer = env::read();
    let public: DeathConstraintPublic = transfer.assert_constraints();
    env::commit(&public);
}
