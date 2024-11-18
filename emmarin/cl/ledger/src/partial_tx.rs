use ledger_proof_statements::ptx::{PtxPrivate, PtxPublic};

use crate::error::{Error, Result};
use cl::zones::*;

pub struct ProvedPartialTx {
    pub ptx: cl::PartialTx,
    pub cm_root: [u8; 32],
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx: &cl::PartialTxWitness,
        input_cm_paths: Vec<Vec<cl::merkle::PathNode>>,
        cm_root: [u8; 32],
        id: ZoneId,
    ) -> Result<ProvedPartialTx> {
        let ptx_private = PtxPrivate {
            ptx: ptx.clone(),
            input_cm_paths,
            cm_root,
            from: id,
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
            ptx: ptx.commit(&id),
            cm_root,
            risc0_receipt: prove_info.receipt,
        })
    }

    pub fn public(&self) -> Result<PtxPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self) -> bool {
        let Ok(proved_ptx_inputs) = self.public() else {
            return false;
        };
        let expected_ptx_inputs = PtxPublic {
            ptx: self.ptx.clone(),
            cm_root: self.cm_root,
        };
        if expected_ptx_inputs != proved_ptx_inputs {
            return false;
        }

        self.risc0_receipt
            .verify(nomos_cl_risc0_proofs::PTX_ID)
            .is_ok()
    }
}
