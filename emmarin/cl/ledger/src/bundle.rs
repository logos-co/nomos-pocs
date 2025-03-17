use crate::tx::ProvedTx;
use cl::crust::{Bundle, BundleWitness};

use hex::FromHex;

#[derive(Debug, Clone)]
pub struct ProvedBundle {
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedBundle {
    pub fn prove(txs: Vec<ProvedTx>) -> Self {
        //show that all ptx's are individually valid, and balance to 0
        let mut env = risc0_zkvm::ExecutorEnv::builder();

        let bundle = BundleWitness {
            txs: txs.iter().map(|tx| tx.public()).collect(),
        };

        for proved_tx in txs {
            env.add_assumption(proved_tx.risc0_receipt);
        }

        let env = env.write(&bundle).unwrap().build().unwrap();

        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, risc0_images::BUNDLE_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'bundle' prover time: {:.2?}, user_cycles: {}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.user_cycles,
            prove_info.stats.total_cycles
        );

        let receipt = prove_info.receipt;

        Self {
            risc0_receipt: receipt,
        }
    }

    pub fn public(&self) -> Bundle {
        self.risc0_receipt.journal.decode().unwrap()
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt
            .verify(<[u8; 32]>::from_hex(risc0_images::BUNDLE_ID).unwrap())
            .is_ok()
    }
}
