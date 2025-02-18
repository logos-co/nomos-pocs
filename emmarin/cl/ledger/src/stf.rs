use cl::mantle::zone::Stf;
use ledger_proof_statements::stf::StfPublic;

#[derive(Debug, Clone)]
pub struct StfProof {
    pub risc0_id: [u32; 8],
    pub public: StfPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

pub fn risc0_stf(risc0_id: [u32; 8]) -> Stf {
    // TODO: hash

    unsafe { core::mem::transmute::<[u32; 8], [u8; 32]>(risc0_id) }
}

impl StfProof {
    pub fn from_risc0(risc0_id: [u32; 8], risc0_receipt: risc0_zkvm::Receipt) -> Self {
        Self {
            risc0_id,
            public: risc0_receipt.journal.decode().unwrap(),
            risc0_receipt,
        }
    }

    pub fn stf(&self) -> Stf {
        risc0_stf(self.risc0_id)
    }
    pub fn verify(&self) -> bool {
        self.risc0_receipt.verify(self.risc0_id).is_ok()
    }

    pub fn nop_stf() -> [u8; 32] {
        risc0_stf(nomos_mantle_risc0_proofs::STF_NOP_ID)
    }

    pub fn prove_nop(public: StfPublic) -> Self {
        let env = risc0_zkvm::ExecutorEnv::builder()
            .write(&public)
            .unwrap()
            .build()
            .unwrap();

        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, nomos_mantle_risc0_proofs::STF_NOP_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'stf' prover time: {:.2?}, user_cycles: {}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.user_cycles,
            prove_info.stats.total_cycles
        );

        let receipt = prove_info.receipt;

        Self {
            risc0_id: nomos_mantle_risc0_proofs::STF_NOP_ID,
            public,
            risc0_receipt: receipt,
        }
    }
}
