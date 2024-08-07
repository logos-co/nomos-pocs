use ledger_proof_statements::death_constraint::DeathConstraintPublic;
use sha2::{Digest, Sha256};

use crate::error::Result;

pub type Risc0DeathConstraintId = [u32; 8];

#[derive(Debug, Clone)]
pub struct DeathProof {
    pub constraint: Risc0DeathConstraintId,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

pub fn risc0_id_to_cl_death_constraint(risc0_id: Risc0DeathConstraintId) -> [u8; 32] {
    // RISC0 proof ids have the format: [u32; 8], and cl death constraint ids have the format [u8; 32].
    // CL death constraints are meaningless beyond being binding, therefore we merely need a collision
    // resisitant mapping of RISC0 ids to cl death constraints.

    let mut hasher = Sha256::new();
    hasher.update(b"NOMOS_RISC0_ID_TO_CL_DEATH_CONSTRAINT");
    for word in risc0_id {
        hasher.update(u32::to_ne_bytes(word));
    }
    let death_constraint: [u8; 32] = hasher.finalize().into();
    death_constraint
}

impl DeathProof {
    pub fn from_risc0(
        risc0_id: Risc0DeathConstraintId,
        risc0_receipt: risc0_zkvm::Receipt,
    ) -> Self {
        Self {
            constraint: risc0_id,
            risc0_receipt,
        }
    }

    pub fn death_commitment(&self) -> cl::DeathCommitment {
        cl::note::death_commitment(&risc0_id_to_cl_death_constraint(self.constraint))
    }

    pub fn public(&self) -> Result<DeathConstraintPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self, expected_public: DeathConstraintPublic) -> bool {
        let Ok(public) = self.public() else {
            return false;
        };

        expected_public == public && self.risc0_receipt.verify(self.constraint).is_ok()
    }

    pub fn nop_constraint() -> [u8; 32] {
        risc0_id_to_cl_death_constraint(nomos_cl_risc0_proofs::DEATH_CONSTRAINT_NOP_ID)
    }

    pub fn prove_nop(nf: cl::Nullifier, ptx_root: cl::PtxRoot) -> Self {
        let death_public = DeathConstraintPublic { nf, ptx_root };
        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&death_public)
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
            .prove_with_opts(env, nomos_cl_risc0_proofs::DEATH_CONSTRAINT_NOP_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'death-nop' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        // extract the receipt.
        let receipt = prove_info.receipt;

        Self::from_risc0(nomos_cl_risc0_proofs::DEATH_CONSTRAINT_NOP_ID, receipt)
    }
}
