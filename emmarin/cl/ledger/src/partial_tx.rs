use ledger_proof_statements::ptx::{PtxPrivate, PtxPublic};

use crate::{
    error::{Error, Result},
    ConstraintProof,
};
use cl::cl::{
    mmr::{MMRProof, MMR},
    PartialTxWitness,
};

#[derive(Debug, Clone)]
pub struct ProvedPartialTx {
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedPartialTx {
    pub fn prove(
        ptx_witness: PartialTxWitness,
        input_cm_proofs: Vec<(MMR, MMRProof)>,
        covenant_proofs: Vec<ConstraintProof>,
    ) -> Result<ProvedPartialTx> {
        let ptx_private = PtxPrivate {
            ptx: ptx_witness,
            input_cm_proofs,
        };

        let mut env = risc0_zkvm::ExecutorEnv::builder();

        for covenant_proof in covenant_proofs {
            env.add_assumption(covenant_proof.risc0_receipt);
        }
        let env = env.write(&ptx_private).unwrap().build().unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_cl_ptx_risc0_proof::PTX_ELF, &opts)
            .map_err(|_| Error::Risc0ProofFailed)?;

        println!(
            "STARK 'ptx' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Ok(Self {
            risc0_receipt: prove_info.receipt,
        })
    }

    pub fn public(&self) -> PtxPublic {
        self.risc0_receipt.journal.decode().unwrap()
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt
            .verify(nomos_cl_ptx_risc0_proof::PTX_ID)
            .is_ok()
    }
}
