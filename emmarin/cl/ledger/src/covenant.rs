use ledger_proof_statements::covenant::{SpendingCovenantPublic, SupplyCovenantPublic};

use crate::error::Result;

macro_rules! impl_covenant_proof {
    ($name:ident, $public:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            pub risc0_id: [u32; 8],
            pub risc0_receipt: risc0_zkvm::Receipt,
        }

        impl $name {
            pub fn from_risc0(risc0_id: [u32; 8], risc0_receipt: risc0_zkvm::Receipt) -> Self {
                Self {
                    risc0_id,
                    risc0_receipt,
                }
            }

            pub fn public(&self) -> Result<$public> {
                Ok(self.risc0_receipt.journal.decode()?)
            }

            pub fn verify(&self, expected_public: $public) -> bool {
                let Ok(public) = self.public() else {
                    return false;
                };

                expected_public == public && self.risc0_receipt.verify(self.risc0_id).is_ok()
            }
        }
    };
}

impl_covenant_proof!(SupplyCovenantProof, SupplyCovenantPublic);
impl_covenant_proof!(SpendingCovenantProof, SpendingCovenantPublic);
