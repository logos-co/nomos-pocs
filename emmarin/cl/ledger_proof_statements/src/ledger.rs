use std::collections::BTreeMap;

use crate::bundle::BundleId;
use crate::bundle::BundlePublic;
use cl::cl::{indexed::BatchUpdateProof, merkle, NoteCommitment};
use cl::zone_layer::{
    ledger::{Ledger, LedgerWitness},
    notes::ZoneId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerProofPublic {
    pub old_ledger: Ledger,
    pub ledger: Ledger,
    pub id: ZoneId,
    pub cross_bundles: Vec<CrossZoneBundle>,
    pub outputs: Vec<NoteCommitment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerProofPrivate {
    pub ledger: LedgerWitness,
    pub id: ZoneId,
    pub bundles: Vec<LedgerBundleWitness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerBundleWitness {
    pub bundle: BundlePublic,
    pub cm_root_proofs: BTreeMap<[u8; 32], merkle::Path>,
    pub nf_proofs: BatchUpdateProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossZoneBundle {
    pub id: BundleId,
    pub zones: Vec<ZoneId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactNullifierProofs {
    pub siblings: Vec<u8>,
    pub paths: Vec<[u8; 32]>,
}

impl CompactNullifierProofs {
    pub fn from_paths(input: Vec<merkle::Path>) -> Self {
        let mut siblings = Vec::with_capacity(input.len());
        let mut paths = Vec::with_capacity(input.len());

        for path in input {
            let mut path_bits = [0u8; 32];
            assert_eq!(path.len(), 64);

            for (i, node) in path.iter().enumerate().rev() {
                match node {
                    merkle::PathNode::Left(sibling) => {
                        siblings.extend(sibling.into_iter());
                    }
                    merkle::PathNode::Right(sibling) => {
                        siblings.extend(sibling.into_iter());
                        set_bit(i as u8, &mut path_bits);
                    }
                }
            }
            paths.push(path_bits);
        }

        Self { siblings, paths }
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

impl IntoIterator for CompactNullifierProofs {
    type Item = merkle::Path;
    type IntoIter = CompactNfIterator;

    fn into_iter(self) -> CompactNfIterator {
        CompactNfIterator {
            siblings: self.siblings,
            paths: self.paths,
        }
    }
}

pub struct CompactNfIterator {
    pub siblings: Vec<u8>,
    pub paths: Vec<[u8; 32]>,
}

impl<'a> Iterator for CompactNfIterator {
    type Item = merkle::Path;

    fn next(&mut self) -> Option<Self::Item> {
        if self.paths.is_empty() {
            return None;
        }

        let path = self.paths.pop().unwrap();

        let mut res = Vec::with_capacity(64);

        for i in 0..=63 {
            if get_bit(i, path) {
                res.push(merkle::PathNode::Right(
                    self.siblings[self.siblings.len() - 32..]
                        .try_into()
                        .unwrap(),
                ))
            } else {
                res.push(merkle::PathNode::Left(
                    self.siblings[self.siblings.len() - 32..]
                        .try_into()
                        .unwrap(),
                ))
            };
            self.siblings.truncate(self.siblings.len() - 32);
        }

        Some(res)
    }
}

fn get_bit(idx: u8, elem: [u8; 32]) -> bool {
    let byte = idx / 8;
    let bit_in_byte = idx - byte * 8;

    (elem[byte as usize] & (1 << bit_in_byte)) != 0
}

fn set_bit(idx: u8, elem: &mut [u8; 32]) {
    let byte = idx / 8;
    let bit_in_byte = idx - byte * 8;

    elem[byte as usize] |= 1 << bit_in_byte;
}
