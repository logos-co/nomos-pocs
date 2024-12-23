use std::collections::BTreeSet;

use crate::cl::merkle;
use lazy_static::lazy_static;

/// absence of element is marked with all 0's
pub static ABSENT: [u8; 32] = [0u8; 32];

/// presence of element is marked with all 1's
pub static PRESENT: [u8; 32] = [255u8; 32];

lazy_static! {
    // the roots of empty merkle trees of diffent heights
    // i.e. all leafs are ABSENT
    static ref EMPTY_ROOTS: [[u8; 32]; 257] = {
        let mut roots = [ABSENT; 257];
        for h in 1..257 {
            roots[h] = merkle::node(roots[h - 1], roots[h - 1]);
        }

        roots
    };
}

pub fn sparse_root(elems: &BTreeSet<[u8; 32]>) -> [u8; 32] {
    sparse_root_rec(0, elems)
}

fn sparse_root_rec(prefix: u64, elems: &BTreeSet<[u8; 32]>) -> [u8; 32] {
    if elems.is_empty() {
        return empty_tree_root(64 - prefix);
    }
    if prefix == 64 {
        assert_eq!(elems.len(), 1);
        return PRESENT;
    }
    // partition the elements
    let (left, right): (BTreeSet<_>, BTreeSet<_>) =
        elems.iter().partition(|e| !bit(prefix as u8, **e));

    merkle::node(
        sparse_root_rec(prefix + 1, &left),
        sparse_root_rec(prefix + 1, &right),
    )
}

pub fn sparse_path(elem: [u8; 32], elems: &BTreeSet<[u8; 32]>) -> Vec<merkle::PathNode> {
    fn sparse_path_rec(
        prefix: u64,
        elem: [u8; 32],
        elems: &BTreeSet<[u8; 32]>,
    ) -> Vec<merkle::PathNode> {
        if prefix == 64 {
            return Vec::new();
        }
        // partition the elements
        let (left, right): (BTreeSet<_>, BTreeSet<_>) =
            elems.iter().partition(|e| !bit(prefix as u8, **e));

        match bit(prefix as u8, elem) {
            true => {
                let left_root = sparse_root_rec(prefix + 1, &left);
                let mut path = sparse_path_rec(prefix + 1, elem, &right);

                path.push(merkle::PathNode::Left(left_root));
                path
            }
            false => {
                let right_root = sparse_root_rec(prefix + 1, &right);
                let mut path = sparse_path_rec(prefix + 1, elem, &left);

                path.push(merkle::PathNode::Right(right_root));
                path
            }
        }
    }

    sparse_path_rec(0, elem, elems)
}

pub fn path_key(path: &[merkle::PathNode]) -> [u8; 32] {
    assert_eq!(path.len(), 64);

    let mut key = [0u8; 32];
    for byte_i in (0..32).rev() {
        let mut byte = 0u8;
        for bit_i in 0..8 {
            byte <<= 1;
            match path[byte_i * 8 + bit_i] {
                merkle::PathNode::Left(_) => byte += 1,
                merkle::PathNode::Right(_) => byte += 0,
            };
        }
        key[31 - byte_i] = byte;
    }

    key
}

fn empty_tree_root(height: u64) -> [u8; 32] {
    assert!(height <= 256);
    EMPTY_ROOTS[height as usize]
}

fn bit(idx: u8, elem: [u8; 32]) -> bool {
    let byte = idx / 8;
    let bit_in_byte = idx - byte * 8;

    (elem[byte as usize] & (1 << bit_in_byte)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_hash() -> [u8; 32] {
        rand::random()
    }

    #[test]
    fn test_neighbour_paths() {
        let elems = BTreeSet::from_iter([[0u8; 32]]);

        let path_0 = sparse_path([0u8; 32], &elems);
        let mut key_1 = [0u8; 32];
        key_1[31] = 128;
        let path_1 = sparse_path(key_1, &elems);

        assert_ne!(path_0, path_1);
    }

    #[test]
    fn test_path_bit_agreement() {
        fn path_bit(idx: u8, path: &[merkle::PathNode]) -> bool {
            match path[255 - idx as usize] {
                merkle::PathNode::Left(_) => true,
                merkle::PathNode::Right(_) => false,
            }
        }

        let key = random_hash();
        let path = sparse_path(key, &BTreeSet::new());

        for i in 0..=255 {
            let b = bit(i, key);
            let pb = path_bit(i, &path);
            assert_eq!(b, pb, "{}!={}@{}", b, pb, i);
        }
    }

    #[test]
    fn test_path_key() {
        let elems = BTreeSet::from_iter(std::iter::repeat_with(random_hash).take(10));

        // membership proofs
        for e in elems.iter() {
            let path = sparse_path(*e, &elems);
            assert_eq!(path_key(&path), *e);
        }

        // non-membership proofs
        for _ in 0..10 {
            let elem = random_hash();
            let path = sparse_path(elem, &elems);
            assert_eq!(path_key(&path), elem);
        }
    }

    #[test]
    fn test_sparse_path() {
        let elems = BTreeSet::from_iter(std::iter::repeat_with(random_hash).take(10));

        let root = sparse_root(&elems);

        // membership proofs
        for e in elems.iter() {
            let path = sparse_path(*e, &elems);
            assert_eq!(merkle::path_root(PRESENT, &path), root);
        }

        // non-membership proofs
        for _ in 0..10 {
            let elem = random_hash();
            let path = sparse_path(elem, &elems);
            assert!(!elems.contains(&elem));
            assert_eq!(merkle::path_root(ABSENT, &path), root);
        }
    }

    #[test]
    fn test_sparse_non_membership_in_empty_tree() {
        let root = sparse_root(&BTreeSet::new());

        let path = sparse_path([0u8; 32], &BTreeSet::new());

        assert_eq!(merkle::path_root(ABSENT, &path), root);

        for (h, node) in path.into_iter().enumerate() {
            match node {
                merkle::PathNode::Left(hash) | merkle::PathNode::Right(hash) => {
                    assert_eq!(hash, empty_tree_root(h as u64))
                }
            }
        }
    }

    #[test]
    fn test_sparse_root_left_most_occupied() {
        let root = sparse_root(&BTreeSet::from_iter([[0u8; 32]]));

        // We are constructing the tree:
        //
        //      / \
        //     / \ 0 subtree
        //    / \ 0 subtree
        //   1  0
        let mut expected_root = PRESENT;
        for h in 0..=255 {
            expected_root = merkle::node(expected_root, empty_tree_root(h))
        }

        assert_eq!(root, expected_root)
    }

    #[test]
    fn test_sparse_root_right_most_occupied() {
        let root = sparse_root(&BTreeSet::from_iter([[255u8; 32]]));

        // We are constructing the tree:
        //
        //  /\
        // 0 /\
        //  0 /\
        //   0 1
        let mut expected_root = PRESENT;
        for h in 0..=255 {
            expected_root = merkle::node(empty_tree_root(h), expected_root)
        }

        assert_eq!(root, expected_root)
    }

    #[test]
    fn test_sparse_root_middle_elem() {
        let elem = {
            let mut x = [255u8; 32];
            x[0] = 254;
            x
        };
        assert!(!bit(0, elem));
        for i in 1..=255 {
            assert!(bit(i, elem));
        }

        let root = sparse_root(&BTreeSet::from_iter([elem]));

        // We are constructing the tree:
        //    root
        //    / \
        //   /\ 0
        //  0 /\
        //   0 /\
        //    0 ...
        //       \
        //       1
        let mut expected_root = PRESENT;
        for h in 0..=254 {
            expected_root = merkle::node(empty_tree_root(h), expected_root)
        }
        expected_root = merkle::node(expected_root, empty_tree_root(255));

        assert_eq!(root, expected_root)
    }

    #[test]
    fn test_sparse_root_middle_weave_elem() {
        let elem = [85u8; 32];
        for i in 0..=255 {
            assert_eq!(bit(i, elem), i % 2 == 0);
        }

        let root = sparse_root(&BTreeSet::from_iter([elem]));

        // We are constructing the tree:
        //  /\
        // 0 /\
        //  /\0
        //  /\
        // 0 /\
        //  /\0
        // 0 1

        let mut expected_root = PRESENT;
        for h in 0..=255 {
            if h % 2 == 0 {
                expected_root = merkle::node(expected_root, empty_tree_root(h))
            } else {
                expected_root = merkle::node(empty_tree_root(h), expected_root)
            }
        }
        assert_eq!(root, expected_root)
    }

    #[test]
    fn test_sparse_multiple_elems() {
        let root = sparse_root(&BTreeSet::from_iter([[0u8; 32], [255u8; 32]]));

        // We are constructing the tree:
        //     root
        //    /  \
        //   /\  /\
        //  /\0 0 /\
        // 1 0   0 1

        let mut left_root = PRESENT;
        for h in 0..=254 {
            left_root = merkle::node(left_root, empty_tree_root(h))
        }

        let mut right_root = PRESENT;
        for h in 0..=254 {
            right_root = merkle::node(empty_tree_root(h), right_root)
        }
        let expected_root = merkle::node(left_root, right_root);

        assert_eq!(root, expected_root)
    }

    #[test]
    fn test_bit() {
        for i in 0..=255 {
            assert!(!bit(i, [0u8; 32]))
        }

        for i in 0..=255 {
            assert!(bit(i, [255u8; 32]))
        }

        for i in 0..=255 {
            assert_eq!(bit(i, [85u8; 32]), i % 2 == 0)
        }
    }
    #[test]
    fn test_empty_tree_root() {
        assert_eq!(empty_tree_root(0), ABSENT);

        assert_eq!(empty_tree_root(1), merkle::node(ABSENT, ABSENT));
        assert_eq!(
            empty_tree_root(2),
            merkle::node(merkle::node(ABSENT, ABSENT), merkle::node(ABSENT, ABSENT)),
        );
        assert_eq!(
            empty_tree_root(3),
            merkle::node(
                merkle::node(merkle::node(ABSENT, ABSENT), merkle::node(ABSENT, ABSENT)),
                merkle::node(merkle::node(ABSENT, ABSENT), merkle::node(ABSENT, ABSENT)),
            )
        );
    }
}
