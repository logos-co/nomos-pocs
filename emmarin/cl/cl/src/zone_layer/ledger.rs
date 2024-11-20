use crate::cl::{merkle, NoteCommitment, Nullifier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    cm_root: [u8; 32],
    nf_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerWitness {
    pub commitments: Vec<NoteCommitment>,
    pub nullifiers: Vec<Nullifier>,
}

const MAX_COMM: usize = 256;
const MAX_NULL: usize = 256;

impl LedgerWitness {
    pub fn commit(&self) -> Ledger {
        Ledger {
            cm_root: self.cm_root(),
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

    pub fn cm_root(&self) -> [u8; 32] {
        let bytes = self
            .commitments
            .iter()
            .map(|i| i.as_bytes().to_vec())
            .collect::<Vec<_>>();
        merkle::root(merkle::padded_leaves::<MAX_COMM>(&bytes))
    }

    pub fn cm_path(&self, cm: &NoteCommitment) -> Option<Vec<merkle::PathNode>> {
        let bytes = self
            .commitments
            .iter()
            .map(|i| i.as_bytes().to_vec())
            .collect::<Vec<_>>();
        let leaves = merkle::padded_leaves::<MAX_COMM>(&bytes);
        let idx = self.commitments.iter().position(|c| c == cm)?;
        Some(merkle::path(leaves, idx))
    }
}
