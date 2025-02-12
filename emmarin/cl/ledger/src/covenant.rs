use cl::crust::{Nullifier, TxRoot};
use ledger_proof_statements::covenant::{SpendingCovenantPublic, SupplyCovenantPublic};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct SupplyCovenantProof {
    pub risc0_id: [u32; 8],
    pub risc0_receipt: risc0_zkvm::Receipt,
}

#[derive(Debug, Clone)]
pub struct SpendingCovenantProof {
    pub risc0_id: [u32; 8],
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl SpendingCovenantProof {
    pub fn from_risc0(risc0_id: [u32; 8], risc0_receipt: risc0_zkvm::Receipt) -> Self {
        Self {
            risc0_id,
            risc0_receipt,
        }
    }

    pub fn public(&self) -> Result<SpendingCovenantPublic> {
        Ok(self.risc0_receipt.journal.decode()?)
    }

    pub fn verify(&self, expected_public: SpendingCovenantPublic) -> bool {
        let Ok(public) = self.public() else {
            return false;
        };

        expected_public == public && self.risc0_receipt.verify(self.risc0_id).is_ok()
    }

    pub fn nop() -> [u8; 32] {
        todo!()
    }

    pub fn prove_nop(nf: Nullifier, tx_root: TxRoot) -> Self {
        let covenant_public = SpendingCovenantPublic { nf, tx_root };
        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&covenant_public)
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
            .prove_with_opts(env, nomos_cl_risc0_proofs::SPENDING_NOP_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'constraint-nop' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        // extract the receipt.
        let receipt = prove_info.receipt;

        Self::from_risc0(nomos_cl_risc0_proofs::SPENDING_NOP_ID, receipt)
    }
}
