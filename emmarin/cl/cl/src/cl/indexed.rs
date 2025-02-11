use super::{
    merkle::{self, leaf, Path, PathNode},
    mmr::{Root, MMR},
    Nullifier,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

// the roots of empty merkle trees of diffent heights
static EMPTY_ROOTS: [[u8; 32]; 32] = [
    [0; 32],
    [
        177, 12, 156, 85, 54, 248, 12, 101, 211, 140, 131, 82, 142, 82, 119, 152, 90, 154, 54, 64,
        122, 123, 61, 166, 144, 13, 169, 27, 61, 69, 25, 49,
    ],
    [
        192, 157, 232, 29, 199, 66, 141, 214, 64, 82, 152, 83, 59, 136, 250, 91, 209, 32, 143, 28,
        190, 109, 233, 43, 28, 58, 90, 240, 214, 89, 0, 157,
    ],
    [
        234, 171, 4, 69, 194, 82, 65, 197, 220, 105, 217, 26, 78, 139, 7, 35, 123, 137, 173, 57,
        224, 85, 154, 88, 114, 245, 145, 12, 58, 131, 158, 126,
    ],
    [
        200, 121, 6, 41, 76, 104, 193, 220, 1, 170, 134, 10, 156, 51, 252, 2, 116, 137, 120, 220,
        198, 203, 132, 233, 175, 242, 212, 37, 237, 112, 220, 85,
    ],
    [
        243, 70, 215, 24, 17, 201, 70, 193, 170, 22, 243, 226, 154, 3, 91, 175, 130, 131, 163, 76,
        238, 174, 153, 166, 34, 53, 59, 177, 188, 93, 88, 109,
    ],
    [
        47, 194, 241, 92, 49, 216, 212, 37, 215, 16, 16, 92, 141, 120, 190, 171, 192, 166, 167, 90,
        241, 16, 216, 221, 137, 26, 189, 228, 22, 8, 29, 230,
    ],
    [
        114, 18, 79, 249, 200, 32, 139, 234, 8, 208, 147, 247, 248, 158, 45, 172, 74, 203, 42, 8,
        111, 32, 54, 21, 41, 79, 254, 184, 180, 21, 124, 74,
    ],
    [
        145, 84, 112, 4, 33, 107, 225, 144, 128, 175, 222, 242, 151, 233, 251, 72, 111, 174, 96,
        156, 47, 199, 103, 138, 225, 136, 122, 77, 113, 155, 234, 247,
    ],
    [
        11, 157, 239, 22, 43, 157, 252, 172, 170, 216, 246, 54, 17, 250, 62, 150, 56, 71, 10, 199,
        73, 149, 210, 55, 128, 177, 66, 3, 53, 117, 251, 183,
    ],
    [
        185, 189, 114, 54, 194, 160, 33, 78, 253, 117, 195, 9, 8, 5, 98, 153, 232, 236, 51, 123,
        149, 89, 219, 121, 144, 24, 131, 23, 133, 185, 43, 84,
    ],
    [
        112, 167, 71, 47, 253, 157, 13, 91, 220, 65, 136, 163, 159, 67, 93, 31, 20, 26, 211, 53, 3,
        87, 214, 79, 139, 91, 175, 186, 241, 96, 36, 50,
    ],
    [
        194, 180, 108, 122, 130, 69, 19, 30, 123, 135, 82, 112, 184, 120, 190, 218, 243, 195, 112,
        62, 233, 93, 50, 163, 17, 113, 50, 116, 204, 0, 154, 48,
    ],
    [
        148, 210, 36, 218, 105, 22, 94, 122, 161, 188, 141, 168, 111, 73, 85, 240, 124, 61, 14,
        224, 230, 127, 232, 216, 62, 226, 15, 241, 178, 214, 74, 146,
    ],
    [
        40, 223, 100, 218, 109, 7, 142, 65, 131, 44, 18, 199, 189, 186, 19, 141, 26, 17, 199, 237,
        175, 131, 246, 119, 240, 208, 9, 158, 20, 61, 123, 78,
    ],
    [
        201, 24, 167, 145, 146, 0, 225, 211, 222, 4, 168, 99, 66, 145, 227, 153, 137, 203, 210, 71,
        159, 65, 73, 114, 68, 95, 197, 195, 252, 157, 176, 136,
    ],
    [
        48, 213, 33, 6, 16, 231, 203, 89, 97, 59, 140, 45, 122, 220, 219, 100, 28, 28, 11, 94, 152,
        121, 73, 81, 17, 43, 221, 62, 168, 253, 60, 75,
    ],
    [
        235, 42, 170, 207, 251, 244, 212, 33, 244, 247, 205, 152, 200, 175, 127, 130, 29, 185, 12,
        168, 155, 181, 186, 70, 143, 116, 118, 125, 213, 61, 133, 216,
    ],
    [
        114, 156, 155, 68, 120, 46, 130, 183, 148, 220, 222, 87, 255, 204, 77, 158, 109, 250, 218,
        97, 85, 113, 90, 210, 38, 127, 1, 108, 150, 234, 218, 8,
    ],
    [
        23, 0, 234, 63, 219, 38, 225, 234, 86, 65, 254, 152, 99, 26, 147, 35, 220, 157, 73, 119,
        125, 42, 230, 7, 31, 193, 194, 14, 3, 66, 238, 182,
    ],
    [
        98, 183, 177, 156, 96, 245, 221, 11, 101, 129, 202, 229, 95, 119, 42, 206, 89, 94, 213,
        165, 7, 78, 36, 88, 2, 102, 137, 50, 212, 33, 228, 222,
    ],
    [
        72, 59, 68, 178, 17, 108, 122, 234, 144, 160, 205, 221, 106, 249, 141, 34, 247, 190, 97,
        192, 237, 171, 37, 251, 238, 87, 249, 236, 210, 120, 99, 114,
    ],
    [
        199, 172, 23, 156, 51, 202, 195, 224, 29, 147, 201, 201, 224, 152, 153, 28, 175, 3, 39, 40,
        14, 98, 231, 38, 117, 171, 80, 6, 102, 236, 107, 67,
    ],
    [
        130, 105, 50, 158, 64, 150, 93, 137, 190, 66, 61, 158, 243, 130, 105, 85, 76, 126, 192,
        139, 131, 236, 181, 34, 227, 186, 123, 81, 124, 83, 236, 53,
    ],
    [
        29, 170, 86, 82, 122, 96, 225, 198, 251, 48, 125, 20, 235, 213, 119, 64, 95, 24, 196, 180,
        170, 18, 173, 51, 243, 126, 249, 126, 222, 136, 100, 29,
    ],
    [
        144, 79, 68, 40, 85, 101, 172, 71, 165, 66, 18, 29, 183, 16, 224, 80, 32, 242, 43, 104,
        247, 113, 196, 87, 107, 148, 111, 209, 145, 145, 193, 172,
    ],
    [
        247, 113, 160, 20, 26, 123, 24, 107, 219, 159, 232, 236, 212, 181, 146, 159, 254, 102, 166,
        103, 141, 17, 38, 106, 73, 250, 12, 56, 18, 126, 253, 59,
    ],
    [
        161, 111, 104, 235, 136, 130, 176, 167, 161, 49, 57, 160, 91, 220, 207, 169, 208, 228, 131,
        64, 251, 123, 30, 207, 135, 64, 14, 80, 39, 91, 44, 30,
    ],
    [
        213, 239, 239, 81, 151, 152, 116, 196, 117, 174, 223, 128, 213, 197, 4, 49, 154, 132, 187,
        96, 86, 68, 237, 185, 223, 205, 118, 91, 158, 98, 202, 176,
    ],
    [
        52, 136, 50, 107, 42, 155, 186, 152, 251, 91, 53, 50, 239, 148, 165, 86, 84, 80, 117, 168,
        142, 47, 181, 177, 49, 210, 235, 228, 6, 189, 23, 175,
    ],
    [
        40, 108, 31, 110, 180, 110, 13, 47, 169, 96, 51, 163, 201, 72, 25, 8, 134, 12, 176, 44,
        221, 250, 108, 225, 154, 236, 208, 26, 170, 126, 80, 12,
    ],
    [
        185, 231, 113, 255, 127, 172, 246, 169, 177, 34, 116, 231, 131, 19, 25, 81, 91, 136, 95,
        192, 80, 179, 134, 27, 205, 18, 151, 234, 202, 116, 165, 249,
    ],
];

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

impl<'a> Iterator for PathIterator<'a> {
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
            assert!(in_gap.len() >= 1, "unused low nf");

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
        assert_eq!(cur_root, frontier_root(&mmr.roots));

        for new_leaf in new_leaves {
            mmr.push(&new_leaf.to_bytes());
        }

        frontier_root(&mmr.roots)
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
    use proptest_macro::property_test;

    #[test]
    fn test_empty_roots() {
        let mut root = [0; 32];
        for i in 0..32 {
            assert_eq!(root, EMPTY_ROOTS[i]);
            root = merkle::node(root, root);
        }
    }

    #[property_test]
    fn test_frontier_root(elems: Vec<[u8; 32]>) {
        let mut mmr = MMR::new();
        for elem in &elems {
            mmr.push(elem);
        }
        assert_eq!(
            frontier_root(&mmr.roots),
            merkle::root(&merkle::padded_leaves(
                &elems
                    .into_iter()
                    .map(|array| array.to_vec())
                    .collect::<Vec<_>>()
            ))
        );
    }

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
