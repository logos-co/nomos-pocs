use cl::merkle;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl MMR {
    pub fn new() -> Self {
        Self { roots: vec![] }
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
        let leaf = merkle::leaf(elem);
        let root = merkle::path_root(leaf, &proof.path);

        for mmr_root in self.roots.iter() {
            if mmr_root.height == (path_len + 1) as u8 {
                return mmr_root.root == root;
            }
        }

        false
    }

    pub fn commit(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for mrr_root in self.roots.iter() {
            hasher.update(mrr_root.root);
            hasher.update(mrr_root.height.to_le_bytes());
        }
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mrr_push() {
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
