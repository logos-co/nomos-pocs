use crate::cl::{
    indexed::{BatchUpdateProof, NullifierTree},
    mmr::{UpdateableMMRProof, MMR},
    NoteCommitment, Nullifier,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    cm_root: [u8; 32],
    nf_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerWitness {
    pub commitments: MMR,
    pub nf_root: [u8; 32],
}

impl LedgerWitness {
    pub fn commit(&self) -> Ledger {
        Ledger {
            cm_root: self.commitments.commit(),
            nf_root: self.nf_root,
        }
    }

    pub fn valid_cm_root(&self, root: [u8; 32]) -> bool {
        self.commitments.roots.iter().any(|r| r.root == root)
    }

    pub fn add_commitment(&mut self, cm: &NoteCommitment) {
        self.commitments.push(&cm.0);
    }

    pub fn assert_nfs_update(&mut self, proof: &BatchUpdateProof) {
        // update the nullifer root with the nullifier inserted into the tree
        self.nf_root = proof.verify(self.nf_root);
    }
}

#[derive(Debug, Default, Clone)]
pub struct LedgerState {
    pub commitments: MMR,
    pub nullifiers: NullifierTree,
}

impl LedgerState {
    pub fn to_witness(&self) -> LedgerWitness {
        LedgerWitness {
            commitments: self.commitments.clone(),
            nf_root: self.nf_root(),
        }
    }

    pub fn nf_root(&self) -> [u8; 32] {
        self.nullifiers.root()
    }

    pub fn add_commitment(&mut self, cm: &NoteCommitment) -> UpdateableMMRProof {
        self.commitments.push_updateable(&cm.0)
    }

    pub fn add_nullifiers(&mut self, nfs: Vec<Nullifier>) -> BatchUpdateProof {
        self.nullifiers.insert_batch(nfs)
    }
}
