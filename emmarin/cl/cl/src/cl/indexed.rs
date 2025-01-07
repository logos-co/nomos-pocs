use super::{
    merkle::{self, leaf, Path},
    mmr::{Root, MMR},
    Nullifier,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

// TODO: make this static otherwise we pay computation costs at runtime
lazy_static::lazy_static! {
    // the roots of empty merkle trees of diffent heights
    // i.e. all leafs are ABSENT
    static ref EMPTY_ROOTS: [[u8; 32]; 32] = {
        let mut roots = [[0; 32]; 32];
        for h in 1..32 {
            roots[h] = merkle::node(roots[h - 1], roots[h - 1]);
        }

        roots
    };
}

#[derive(Default, Debug, Clone)]
pub struct NullifierTree {
    leaves: Vec<Leaf>,
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
        merkle::padded_leaves(&self.leaves.iter().map(|l| l.to_bytes()).collect::<Vec<_>>())
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
        assert_eq!(self.root(), frontier_root(&mmr.roots));

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
        assert_eq!(self.root(), frontier_root(&mmr.roots));

        for new_leaf in new_leaves {
            self.leaves.push(new_leaf);
        }

        BatchUpdateProof {
            values,
            low_nfs,
            low_nf_paths,
            mmr,
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Leaf {
    value: Nullifier,
    next_value: Nullifier,
}

impl Leaf {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.value
            .0
            .into_iter()
            .chain(self.next_value.0.into_iter())
            .collect()
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
        assert_eq!(updated_root, frontier_root(&self.mmr.roots));

        let mut mmr = self.mmr.clone();
        mmr.push(&new_leaf.to_bytes());

        frontier_root(&mmr.roots)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateProof {
    values: Vec<Nullifier>,
    low_nfs: Vec<Leaf>,
    low_nf_paths: Vec<Path>,
    mmr: MMR,
}

impl BatchUpdateProof {
    pub fn nullifiers(&self) -> &[Nullifier] {
        &self.values
    }

    pub fn verify(&self, old_root: [u8; 32]) -> [u8; 32] {
        // TODO: old compiter in risc0, use is_sorted
        for window in self.values.windows(2) {
            assert!(window[0] < window[1]);
        }

        let mut new_leaves = Vec::new();
        let mut cur_root = old_root;
        let mut values = self.values.iter();

        for (low_nf, path) in self.low_nfs.iter().zip(&self.low_nf_paths) {
            let in_gap = values
                .peeking_take_while(|v| in_interval(*low_nf, **v))
                .copied()
                .collect::<Vec<_>>();
            assert!(in_gap.len() > 1, "unused low nf");

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

            assert_eq!(cur_root, merkle::path_root(leaf(&low_nf.to_bytes()), path));
            cur_root = merkle::path_root(leaf(&updated_low_nf.to_bytes()), path);
        }

        assert_eq!(cur_root, frontier_root(&self.mmr.roots));

        let mut mmr = self.mmr.clone();
        for new_leaf in new_leaves {
            mmr.push(&new_leaf.to_bytes());
        }

        frontier_root(&mmr.roots)
    }
}

fn in_interval(low_nf: Leaf, value: Nullifier) -> bool {
    low_nf.value < value && value < low_nf.next_value
}

fn frontier_root(roots: &[Root]) -> [u8; 32] {
    if roots.is_empty() {
        return EMPTY_ROOTS[0];
    }
    if roots.len() == 1 {
        return roots[0].root;
    }
    let mut root = EMPTY_ROOTS[0];
    let mut depth = 1;
    for last in roots.iter().rev() {
        while depth < last.height {
            root = merkle::node(root, EMPTY_ROOTS[depth as usize - 1]);
            depth += 1;
        }
        root = merkle::node(last.root, root);
        depth += 1;
    }

    root
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

        let proof = tree_batch.insert_batch(values);

        assert_eq!(
            proof.verify(NullifierTree::new().root()),
            tree_single.root()
        );
    }
}
