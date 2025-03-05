use cl::crust::BundleWitness;
use hex::FromHex;
use risc0_zkvm::{guest::env, serde};

fn main() {
    let bundle_private: BundleWitness = env::read();

    for tx in &bundle_private.txs {
        env::verify(
            <[u8; 32]>::from_hex(risc0_images::BUNDLE_ID).unwrap(),
            &serde::to_vec(&tx).unwrap(),
        )
        .unwrap();
    }

    env::commit(&bundle_private.commit());
}
