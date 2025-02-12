use crate::merkle;
use crate::{Digest, Hash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MMR {
    pub roots: Vec<Root>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Root {
    pub root: [u8; 32],
    pub height: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MMRProof {
    pub path: Vec<merkle::PathNode>,
}

impl MMRProof {
    pub fn root(&self, elem: &[u8]) -> [u8; 32] {
        let leaf = merkle::leaf(elem);
        merkle::path_root(leaf, &self.path)
    }
}

impl MMR {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, elem: &[u8]) -> MMRProof {
        let new_root = Root {
            root: merkle::leaf(elem),
            height: 1,
        };
        self.roots.push(new_root);

        let mut path = vec![];

        for i in (1..self.roots.len()).rev() {
            if self.roots[i].height == self.roots[i - 1].height {
                path.push(merkle::PathNode::Left(self.roots[i - 1].root));

                self.roots[i - 1] = Root {
                    root: merkle::node(self.roots[i - 1].root, self.roots[i].root),
                    height: self.roots[i - 1].height + 1,
                };

                self.roots.remove(i);
            } else {
                break;
            }
        }

        MMRProof { path }
    }

    pub fn verify_proof(&self, elem: &[u8], proof: &MMRProof) -> bool {
        let path_len = proof.path.len();
        let root = proof.root(elem);

        for mmr_root in self.roots.iter() {
            if mmr_root.height == (path_len + 1) as u8 {
                return mmr_root.root == root;
            }
        }

        false
    }

    pub fn commit(&self) -> [u8; 32] {
        // todo: baggin the peaks
        let mut hasher = Hash::new();
        for mmr_root in self.roots.iter() {
            hasher.update(mmr_root.root);
            hasher.update(mmr_root.height.to_le_bytes());
        }
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mmr_push() {
        let mut mmr = MMR::new();
        let proof = mmr.push(b"hello");

        assert_eq!(mmr.roots.len(), 1);
        assert_eq!(mmr.roots[0].height, 1);
        assert_eq!(mmr.roots[0].root, merkle::leaf(b"hello"));
        assert!(mmr.verify_proof(b"hello", &proof));

        let proof = mmr.push(b"world");

        assert_eq!(mmr.roots.len(), 1);
        assert_eq!(mmr.roots[0].height, 2);
        assert_eq!(
            mmr.roots[0].root,
            merkle::node(merkle::leaf(b"hello"), merkle::leaf(b"world"))
        );
        assert!(mmr.verify_proof(b"world", &proof));

        let proof = mmr.push(b"!");

        assert_eq!(mmr.roots.len(), 2);
        assert_eq!(mmr.roots[0].height, 2);
        assert_eq!(
            mmr.roots[0].root,
            merkle::node(merkle::leaf(b"hello"), merkle::leaf(b"world"))
        );
        assert_eq!(mmr.roots[1].height, 1);
        assert_eq!(mmr.roots[1].root, merkle::leaf(b"!"));
        assert!(mmr.verify_proof(b"!", &proof));

        let proof = mmr.push(b"!");

        assert_eq!(mmr.roots.len(), 1);
        assert_eq!(mmr.roots[0].height, 3);
        assert_eq!(
            mmr.roots[0].root,
            merkle::node(
                merkle::node(merkle::leaf(b"hello"), merkle::leaf(b"world")),
                merkle::node(merkle::leaf(b"!"), merkle::leaf(b"!"))
            )
        );
        assert!(mmr.verify_proof(b"!", &proof));
    }
}
