use crate::error::{Error, Result};
use cl::cl::{
    merkle,
    pact::{Pact, PactWitness},
};
use ledger_proof_statements::pact::{PactPrivate, PactPublic};

#[derive(Debug, Clone)]
pub struct ProvedPact {
    pub pact: Pact,
    pub cm_root: [u8; 32],
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedPact {
    pub fn prove(
        pact: PactWitness,
        input_cm_paths: Vec<Vec<merkle::PathNode>>,
        cm_root: [u8; 32],
    ) -> Result<ProvedPact> {
        let pact_private = PactPrivate {
            pact,
            input_cm_paths,
            cm_root,
        };

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&pact_private)
            .unwrap()
            .build()
            .unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_risc0_proofs::PACT_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'pact' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Ok(Self {
            pact: pact_private.pact.commit(),
            cm_root,
            risc0_receipt: prove_info.receipt,
        })
    }

    pub fn public(&self) -> Result<PactPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(proved_ptx_inputs) = self.public() else {
            return false;
        };
        let expected_ptx_inputs = PactPublic {
            pact: self.pact.clone(),
            cm_root: self.cm_root,
        };
        if expected_ptx_inputs != proved_ptx_inputs {
            return false;
        }

        self.risc0_receipt
            .verify(nomos_cl_risc0_proofs::PACT_ID)
            .is_ok()
    }
}
