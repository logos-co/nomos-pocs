use cl::crust::BundleWitness;
use risc0_zkvm::{guest::env, serde};

fn main() {
    let bundle_private: BundleWitness = env::read();

    for tx in &bundle_private.txs {
        env::verify(nomos_cl_tx_risc0_proof::TX_ID, &serde::to_vec(&tx).unwrap()).unwrap();
    }

    env::commit(&bundle_private.commit());
}
