use crate::ds::merkle;
use crate::{Digest, Hash};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MMR {
    pub roots: Vec<Root>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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

    pub fn frontier_root(&self) -> [u8; 32] {
        if self.roots.is_empty() {
            return EMPTY_ROOTS[0];
        }
        if self.roots.len() == 1 {
            return self.roots[0].root;
        }
        let mut root = EMPTY_ROOTS[0];
        let mut depth = 1;
        for last in self.roots.iter().rev() {
            while depth < last.height {
                root = merkle::node(root, EMPTY_ROOTS[depth as usize - 1]);
                depth += 1;
            }
            root = merkle::node(last.root, root);
            depth += 1;
        }

        root
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest_macro::property_test;

    #[test]
    fn test_empty_roots() {
        assert_eq!(EMPTY_ROOTS.len(), 32);

        let mut root = [0; 32];
        for expected_root in EMPTY_ROOTS {
            assert_eq!(root, expected_root);
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
            mmr.frontier_root(),
            merkle::root(&merkle::padded_leaves(elems))
        );
    }

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
