use goas_proof_statements::user_note::UserAtomicTransfer;
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let transfer: UserAtomicTransfer = env::read();
    let block_height: u64 = env::read();
    let public: ConstraintPublic = transfer.assert_constraints(block_height);
    env::commit(&public);
}
