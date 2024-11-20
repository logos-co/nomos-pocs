use cl::zone_layer::notes::Stf;
use ledger_proof_statements::stf::StfPublic;

#[derive(Debug, Clone)]
pub struct StfProof {
    pub risc0_id: [u32; 8],
    pub public: StfPublic,
    pub risc0_receipt: risc0_zkvm::Receipt,
}

pub fn risc0_constraint(risc0_id: [u32; 8]) -> Stf {
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
        risc0_constraint(self.risc0_id)
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt.verify(self.risc0_id).is_ok()
    }
}
