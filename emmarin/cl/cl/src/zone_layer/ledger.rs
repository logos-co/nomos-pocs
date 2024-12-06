use std::collections::BTreeSet;

use crate::cl::{
    merkle,
    mmr::{MMRProof, MMR},
    sparse_merkle, NoteCommitment, Nullifier,
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

    pub fn assert_nf_update(&mut self, nf: Nullifier, path: &[merkle::PathNode]) {
        // verify that the path corresponds to the nullifier
        assert_eq!(sparse_merkle::path_key(path), nf.0);

        // verify that the nullifier was not already present
        assert_eq!(merkle::path_root(sparse_merkle::ABSENT, path), self.nf_root);

        // update the nullifer root with the nullifier inserted into the tree
        self.nf_root = merkle::path_root(sparse_merkle::PRESENT, path);
    }
}

pub struct LedgerState {
    commitments: MMR,
    nullifiers: BTreeSet<[u8; 32]>,
}

impl LedgerState {
    pub fn to_witness(&self) -> LedgerWitness {
        LedgerWitness {
            commitments: self.commitments.clone(),
            nf_root: self.nf_root(),
        }
    }

    pub fn nf_root(&self) -> [u8; 32] {
        sparse_merkle::sparse_root(&self.nullifiers)
    }

    pub fn add_commitment(&mut self, cm: NoteCommitment) -> MMRProof {
        self.commitments.push(&cm.0)
    }

    pub fn add_nullifier(&mut self, nf: Nullifier) -> Vec<merkle::PathNode> {
        let path = sparse_merkle::sparse_path(nf.0, &self.nullifiers);

        assert!(!self.nullifiers.contains(&nf.0));
        self.nullifiers.insert(nf.0);

        path
    }
}
