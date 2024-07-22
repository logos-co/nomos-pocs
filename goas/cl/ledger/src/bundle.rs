use crate::error::Result;

pub struct ProvedBundle {
    pub bundle: cl::Bundle,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedBundle {
    pub fn prove(bundle: &cl::Bundle, bundle_witness: &cl::BundleWitness) -> Self {
        // need to show that bundle is balanced.
        // i.e. the sum of ptx balances is 0

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&bundle_witness)
            .unwrap()
            .build()
            .unwrap();

        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_risc0_proofs::BUNDLE_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'bundle' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        let receipt = prove_info.receipt;

        Self {
            bundle: bundle.clone(),
            risc0_receipt: receipt,
        }
    }

    pub fn public(&self) -> Result<cl::Balance> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(zero_commitment) = self.public() else {
            return false;
        };

        self.bundle.balance() == zero_commitment
            && self
                .risc0_receipt
                .verify(nomos_cl_risc0_proofs::BUNDLE_ID)
                .is_ok()
    }
}
