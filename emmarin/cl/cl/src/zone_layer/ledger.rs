use crate::cl::{merkle, mmr::MMR, Nullifier};
use serde::{Deserialize, Serialize};

const MAX_NULL: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    cm_root: [u8; 32],
    nf_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerWitness {
    pub commitments: MMR,
    pub nullifiers: Vec<Nullifier>,
}

impl LedgerWitness {
    pub fn commit(&self) -> Ledger {
        Ledger {
            cm_root: self.commitments.commit(),
            nf_root: self.nf_root(),
        }
    }

    pub fn nf_root(&self) -> [u8; 32] {
        let bytes = self
            .nullifiers
            .iter()
            .map(|i| i.as_bytes().to_vec())
            .collect::<Vec<_>>();
        merkle::root(merkle::padded_leaves::<MAX_NULL>(&bytes))
    }
}
