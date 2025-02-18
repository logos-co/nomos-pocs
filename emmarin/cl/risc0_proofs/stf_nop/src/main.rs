use ledger_proof_statements::stf::StfPublic;
use risc0_zkvm::guest::env;

fn main() {
    let public: StfPublic = env::read();
    env::commit(&public);
}
