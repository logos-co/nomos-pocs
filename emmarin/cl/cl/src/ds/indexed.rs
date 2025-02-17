// TODO: generalize this IMT to support arbitrary ordered elements not just nullifiers

use crate::{
    crust::Nullifier,
    ds::merkle::{self, leaf, Path, PathNode},
    ds::mmr::{Root, MMR},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct NullifierTree {
    leaves: Vec<Leaf>,
}

impl Default for NullifierTree {
    fn default() -> Self {
        Self::new()
    }
}

impl NullifierTree {
    pub fn new() -> Self {
        Self {
            leaves: vec![Leaf {
                value: Nullifier([0; 32]),
                next_value: Nullifier([255; 32]),
            }],
        }
    }

    fn hashed_leaves(&self) -> Vec<[u8; 32]> {
        merkle::padded_leaves(self.leaves.iter().map(|l| l.to_bytes()))
    }

    pub fn root(&self) -> [u8; 32] {
        merkle::root(&self.hashed_leaves())
    }

    pub fn insert(&mut self, value: Nullifier) -> UpdateProof {
        let (idx, &low_nf) = self
            .leaves
            .iter()
            .enumerate()
            .find(|(_, l)| in_interval(**l, value))
            .expect("element already exist");
        let low_nf_path = merkle::path(&self.hashed_leaves(), idx);

        let new_leaf = Leaf {
            value,
            next_value: low_nf.next_value,
        };
        self.leaves[idx].next_value = value;

        let mut mmr = MMR::new();
        for leaf in &self.leaves {
            mmr.push(&leaf.to_bytes());
        }
        assert_eq!(self.root(), mmr.frontier_root());

        self.leaves.push(new_leaf);

        UpdateProof {
            value,
            low_nf_path,
            low_nf,
            mmr,
        }
    }

    pub fn insert_batch(&mut self, mut values: Vec<Nullifier>) -> BatchUpdateProof {
        values.sort();

        let mut low_nfs_idx = <BTreeMap<_, Vec<_>>>::new();

        for value in &values {
            let idx = self
                .leaves
                .iter()
                .enumerate()
                .find(|(_, l)| in_interval(**l, *value))
                .expect("element already exist")
                .0;
            low_nfs_idx.entry(idx).or_default().push(*value);
        }

        let mut new_leaves = Vec::new();
        let mut low_nf_paths = Vec::new();
        let mut low_nfs = Vec::new();
        for (idx, values) in &low_nfs_idx {
            let low_nf = self.leaves[*idx];
            low_nfs.push(low_nf);
            low_nf_paths.push(merkle::path(&self.hashed_leaves(), *idx));
            self.leaves[*idx].next_value = *values.first().unwrap();
            for w in values.windows(2) {
                let prev = w[0];
                let next = w[1];
                new_leaves.push(Leaf {
                    value: prev,
                    next_value: next,
                });
            }
            new_leaves.push(Leaf {
                value: *values.last().unwrap(),
                next_value: low_nf.next_value,
            });
        }

        let mut mmr = MMR::new();
        for leaf in &self.leaves {
            mmr.push(&leaf.to_bytes());
        }

        assert_eq!(self.root(), mmr.frontier_root());

        for new_leaf in new_leaves {
            self.leaves.push(new_leaf);
        }

        BatchUpdateProofInner {
            low_nfs,
            low_nf_paths,
            mmr,
        }
        .serialize()
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Leaf {
    value: Nullifier,
    next_value: Nullifier,
}

impl Leaf {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.value.0.into_iter().chain(self.next_value.0).collect()
    }
}

#[derive(Debug, Clone)]
pub struct UpdateProof {
    value: Nullifier,
    low_nf_path: Path,
    low_nf: Leaf,
    mmr: MMR,
}

impl UpdateProof {
    pub fn verify(&self, old_root: [u8; 32]) -> [u8; 32] {
        assert!(in_interval(self.low_nf, self.value));

        assert_eq!(
            merkle::path_root(leaf(&self.low_nf.to_bytes()), &self.low_nf_path),
            old_root
        );

        let new_leaf = Leaf {
            value: self.value,
            next_value: self.low_nf.next_value,
        };

        let mut updated_low_nf = self.low_nf;
        updated_low_nf.next_value = self.value;

        let updated_root = merkle::path_root(leaf(&updated_low_nf.to_bytes()), &self.low_nf_path);
        assert_eq!(updated_root, self.mmr.frontier_root());

        let mut mmr = self.mmr.clone();
        mmr.push(&new_leaf.to_bytes());

        mmr.frontier_root()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchUpdateProofInner {
    low_nfs: Vec<Leaf>,
    low_nf_paths: Vec<Path>,
    mmr: MMR,
}

/// Custom zero-copyish deserialization is needed for decent performance
/// in risc0
#[derive(Debug, Clone)]
pub struct BatchUpdateProof {
    data: Vec<u8>,
}

struct LowNfIterator<'a> {
    data: &'a [u8],
    path_len: usize,
}

#[derive(Debug, Clone)]
struct PathIterator<'p> {
    path: &'p [u8],
}

impl Iterator for PathIterator<'_> {
    type Item = PathNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.is_empty() {
            return None;
        }

        let (node, rest) = self.path.split_at(33);
        self.path = rest;
        match node[0] {
            0 => Some(PathNode::Left(node[1..].try_into().unwrap())),
            1 => Some(PathNode::Right(node[1..].try_into().unwrap())),
            _ => panic!("invalid path node"),
        }
    }
}

impl<'a> Iterator for LowNfIterator<'a> {
    type Item = (Leaf, PathIterator<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        let (low_nf, rest) = self.data.split_at(64 + self.path_len * 33);
        self.data = rest;
        let path = PathIterator {
            path: &low_nf[64..],
        };
        let low_nf = Leaf {
            value: Nullifier(low_nf[..32].try_into().unwrap()),
            next_value: Nullifier(low_nf[32..64].try_into().unwrap()),
        };

        Some((low_nf, path))
    }
}

impl BatchUpdateProof {
    pub fn from_raw_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    fn verify_batch_proof(
        values: &[Nullifier],
        low_nfs: LowNfIterator,
        mut mmr: MMR,
        old_root: [u8; 32],
    ) -> [u8; 32] {
        if values.is_empty() {
            return old_root;
        }

        // TODO: old compiler in risc0, use is_sorted
        for window in values.windows(2) {
            assert!(window[0] < window[1]);
        }

        let mut new_leaves = Vec::new();
        let mut cur_root = old_root;

        let mut values = values.iter();

        for (low_nf, path) in low_nfs {
            let in_gap = values
                .peeking_take_while(|v| in_interval(low_nf, **v))
                .copied()
                .collect::<Vec<_>>();
            assert!(!in_gap.is_empty(), "unused low nf");

            for w in in_gap.windows(2) {
                new_leaves.push(Leaf {
                    value: w[0],
                    next_value: w[1],
                });
            }

            new_leaves.push(Leaf {
                value: *in_gap.last().unwrap(),
                next_value: low_nf.next_value,
            });

            let updated_low_nf = Leaf {
                value: low_nf.value,
                next_value: in_gap[0],
            };

            assert_eq!(
                cur_root,
                merkle::path_root(leaf(&low_nf.to_bytes()), path.clone())
            );
            cur_root = merkle::path_root(leaf(&updated_low_nf.to_bytes()), path);
        }

        assert!(values.next().is_none(), "unused values");
        assert_eq!(cur_root, mmr.frontier_root());

        for new_leaf in new_leaves {
            mmr.push(&new_leaf.to_bytes());
        }

        mmr.frontier_root()
    }

    pub fn verify(&self, nfs: &[Nullifier], old_root: [u8; 32]) -> [u8; 32] {
        if self.data.is_empty() {
            return old_root;
        }
        let len = u32::from_le_bytes(self.data[..4].try_into().unwrap()) as usize;
        let path_len = u32::from_le_bytes(self.data[4..8].try_into().unwrap()) as usize;
        let low_nf_iterator_end = 8 + (path_len * 33 + 64) * len;
        let low_nfs = LowNfIterator {
            data: &self.data[8..low_nf_iterator_end],
            path_len,
        };
        let mut roots = Vec::new();
        for root in self.data[low_nf_iterator_end..].chunks_exact(33) {
            roots.push(Root {
                root: root[1..].try_into().unwrap(),
                height: root[0],
            });
        }
        Self::verify_batch_proof(nfs, low_nfs, MMR { roots }, old_root)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl BatchUpdateProofInner {
    fn serialize(&self) -> BatchUpdateProof {
        if self.low_nfs.is_empty() {
            return BatchUpdateProof { data: Vec::new() };
        }
        let mut data = Vec::new();
        data.extend_from_slice(&(self.low_nfs.len() as u32).to_le_bytes());
        let path_lenghts = self.low_nf_paths[0].len();
        data.extend_from_slice(&(path_lenghts as u32).to_le_bytes());
        for (low_nf, path) in self.low_nfs.iter().zip(&self.low_nf_paths) {
            data.extend_from_slice(&low_nf.to_bytes());
            assert_eq!(path.len(), path_lenghts);
            for node in path {
                match node {
                    merkle::PathNode::Left(sibling) => {
                        data.push(0);
                        data.extend_from_slice(sibling);
                    }
                    merkle::PathNode::Right(sibling) => {
                        data.push(1);
                        data.extend_from_slice(sibling);
                    }
                }
            }
        }

        for root in &self.mmr.roots {
            data.push(root.height);
            data.extend_from_slice(&root.root);
        }

        BatchUpdateProof { data }
    }
}

fn in_interval(low_nf: Leaf, value: Nullifier) -> bool {
    low_nf.value < value && value < low_nf.next_value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_insert_existing() {
        let mut tree = NullifierTree::new();
        tree.insert(Nullifier([0; 32]));
    }

    #[test]
    #[should_panic]
    fn test_insert_existing_batch() {
        let mut tree = NullifierTree::new();
        tree.insert_batch(vec![Nullifier([0; 32])]);
    }

    #[test]
    fn test_insert() {
        let mut tree = NullifierTree::new();
        let proof = tree.insert(Nullifier([1; 32]));

        assert_eq!(proof.verify(NullifierTree::new().root()), tree.root());
    }

    #[test]
    fn test_insert_batch() {
        let mut tree_single = NullifierTree::new();
        let mut tree_batch = NullifierTree::new();
        let values = vec![
            Nullifier([1; 32]),
            Nullifier([2; 32]),
            Nullifier([3; 32]),
            Nullifier([4; 32]),
            Nullifier([5; 32]),
        ];

        for value in &values {
            let old_root = tree_single.root();
            tree_single.insert(*value).verify(old_root);
        }

        let proof = tree_batch.insert_batch(values.clone());

        assert_eq!(
            proof.verify(&values, NullifierTree::new().root()),
            tree_single.root()
        );
    }
}
