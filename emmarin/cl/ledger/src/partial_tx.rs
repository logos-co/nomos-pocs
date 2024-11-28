use ledger_proof_statements::ptx::{PtxPrivate, PtxPublic};

use crate::error::{Error, Result};
use cl::cl::{merkle, PartialTxWitness};

#[derive(Debug, Clone)]
pub struct ProvedPartialTx {
    pub public: PtxPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx_witness: PartialTxWitness,
        input_cm_paths: Vec<Vec<merkle::PathNode>>,
        cm_roots: Vec<[u8; 32]>,
    ) -> Result<ProvedPartialTx> {
        let ptx_private = PtxPrivate {
            ptx: ptx_witness,
            input_cm_paths,
            cm_roots: cm_roots.clone(),
        };

        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&ptx_private)
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
            .prove_with_opts(env, nomos_cl_risc0_proofs::PTX_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'ptx' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Ok(Self {
            public: prove_info.receipt.journal.decode()?,
            risc0_receipt: prove_info.receipt,
        })
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt
            .verify(nomos_cl_risc0_proofs::PTX_ID)
            .is_ok()
    }
}
