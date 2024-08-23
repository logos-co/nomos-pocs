use ledger_proof_statements::bundle::BundlePrivate;

use crate::error::{Error, Result};

pub struct ProvedBundle {
    pub bundle: cl::Bundle,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedBundle {
    pub fn prove(bundle_witness: &cl::BundleWitness) -> Result<Self> {
        // need to show that bundle is balanced.
        // i.e. the sum of ptx balances is 0

        let bundle_private = BundlePrivate {
            balances: bundle_witness
                .partials
                .iter()
                .map(|ptx| ptx.balance())
                .collect(),
        };

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&bundle_private)
            .unwrap()
            .build()
            .unwrap();

        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_risc0_proofs::BUNDLE_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'bundle' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        let receipt = prove_info.receipt;

        Ok(Self {
            bundle: bundle_witness.commit(),
            risc0_receipt: receipt,
        })
    }

    pub fn public(&self) -> Result<ledger_proof_statements::bundle::BundlePublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(bundle_public) = self.public() else {
            return false;
        };

        Vec::from_iter(self.bundle.partials.iter().map(|ptx| ptx.balance)) == bundle_public.balances
            && self
                .risc0_receipt
                .verify(nomos_cl_risc0_proofs::BUNDLE_ID)
                .is_ok()
    }
}
