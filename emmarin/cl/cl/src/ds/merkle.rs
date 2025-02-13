use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

pub fn padded_leaves(elements: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Vec<[u8; 32]> {
    let mut leaves = Vec::from_iter(elements.into_iter().map(|e| leaf(e.as_ref())));
    let pad = leaves.len().next_power_of_two() - leaves.len();
    leaves.extend(std::iter::repeat([0; 32]).take(pad));
    leaves
}

pub fn leaf(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"NOMOS_MERKLE_LEAF");
    hasher.update(data);
    hasher.finalize().into()
}

pub fn node(a: impl AsRef<[u8]>, b: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"NOMOS_MERKLE_NODE");
    hasher.update(a);
    hasher.update(b);
    hasher.finalize().into()
}

pub fn root(elements: &[[u8; 32]]) -> [u8; 32] {
    let n = elements.len();

    assert!(n.is_power_of_two());

    let mut nodes = elements.to_vec();

    for h in (1..=n.ilog2()).rev() {
        for i in 0..2usize.pow(h - 1) {
            nodes[i] = node(nodes[i * 2], nodes[i * 2 + 1]);
        }
    }

    nodes[0]
}

pub type Path = Vec<PathNode>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathNode {
    Left([u8; 32]),
    Right([u8; 32]),
}

pub fn path_root<'a>(leaf: [u8; 32], path: impl IntoIterator<Item: Borrow<PathNode>>) -> [u8; 32] {
    let mut computed_hash = leaf;

    for path_node in path.into_iter() {
        match path_node.borrow() {
            PathNode::Left(sibling_hash) => {
                computed_hash = node(sibling_hash, computed_hash);
            }
            PathNode::Right(sibling_hash) => {
                computed_hash = node(computed_hash, sibling_hash);
            }
        }
    }

    computed_hash
}

pub fn path(leaves: &[[u8; 32]], idx: usize) -> Path {
    assert!(leaves.len().is_power_of_two());
    assert!(idx < leaves.len());
    let max_height = leaves.len().ilog2();

    let mut nodes = leaves.to_vec();
    let mut path = Vec::new();
    let mut idx = idx;

    for h in (1..=max_height).rev() {
        if idx % 2 == 0 {
            path.push(PathNode::Right(nodes[idx + 1]));
        } else {
            path.push(PathNode::Left(nodes[idx - 1]));
        }

        idx /= 2;

        for i in 0..2usize.pow(h - 1) {
            nodes[i] = node(nodes[i * 2], nodes[i * 2 + 1]);
        }
    }

    path
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_root_height_1() {
        let r = root(&padded_leaves(["sand"]));

        let expected = leaf(b"sand");

        assert_eq!(r, expected);
    }

    #[test]
    fn test_root_height_2() {
        let r = root(&padded_leaves(["desert", "sand"]));

        let expected = node(leaf(b"desert"), leaf(b"sand"));

        assert_eq!(r, expected);
    }

    #[test]
    fn test_root_height_3() {
        let r = root(&padded_leaves(["desert", "sand", "feels", "warm"]));

        let expected = node(
            node(leaf(b"desert"), leaf(b"sand")),
            node(leaf(b"feels"), leaf(b"warm")),
        );

        assert_eq!(r, expected);
    }

    #[test]
    fn test_root_height_4() {
        let r = root(&padded_leaves([
            "desert", "sand", "feels", "warm", "at", "night",
        ]));

        let expected = node(
            node(
                node(leaf(b"desert"), leaf(b"sand")),
                node(leaf(b"feels"), leaf(b"warm")),
            ),
            node(
                node(leaf(b"at"), leaf(b"night")),
                node([0u8; 32], [0u8; 32]),
            ),
        );

        assert_eq!(r, expected);
    }

    #[test]
    fn test_path_height_1() {
        let leaves = padded_leaves(["desert"]);
        let r = root(&leaves);

        let p = path(&leaves, 0);
        let expected = vec![];
        assert_eq!(p, expected);
        assert_eq!(path_root(leaf(b"desert"), &p), r);
    }

    #[test]
    fn test_path_height_2() {
        let leaves = padded_leaves(["desert", "sand"]);
        let r = root(&leaves);

        // --- proof for element at idx 0

        let p0 = path(&leaves, 0);
        let expected0 = vec![PathNode::Right(leaf(b"sand"))];
        assert_eq!(p0, expected0);
        assert_eq!(path_root(leaf(b"desert"), &p0), r);

        // --- proof for element at idx 1

        let p1 = path(&leaves, 1);
        let expected1 = vec![PathNode::Left(leaf(b"desert"))];
        assert_eq!(p1, expected1);
        assert_eq!(path_root(leaf(b"sand"), &p1), r);
    }

    #[test]
    fn test_path_height_3() {
        let leaves = padded_leaves(["desert", "sand", "feels", "warm"]);
        let r = root(&leaves);

        // --- proof for element at idx 0

        let p0 = path(&leaves, 0);
        let expected0 = vec![
            PathNode::Right(leaf(b"sand")),
            PathNode::Right(node(leaf(b"feels"), leaf(b"warm"))),
        ];
        assert_eq!(p0, expected0);
        assert_eq!(path_root(leaf(b"desert"), &p0), r);

        // --- proof for element at idx 1

        let p1 = path(&leaves, 1);
        let expected1 = vec![
            PathNode::Left(leaf(b"desert")),
            PathNode::Right(node(leaf(b"feels"), leaf(b"warm"))),
        ];
        assert_eq!(p1, expected1);
        assert_eq!(path_root(leaf(b"sand"), &p1), r);

        // --- proof for element at idx 2

        let p2 = path(&leaves, 2);
        let expected2 = vec![
            PathNode::Right(leaf(b"warm")),
            PathNode::Left(node(leaf(b"desert"), leaf(b"sand"))),
        ];
        assert_eq!(p2, expected2);
        assert_eq!(path_root(leaf(b"feels"), &p2), r);

        // --- proof for element at idx 3

        let p3 = path(&leaves, 3);
        let expected3 = vec![
            PathNode::Left(leaf(b"feels")),
            PathNode::Left(node(leaf(b"desert"), leaf(b"sand"))),
        ];
        assert_eq!(p3, expected3);
        assert_eq!(path_root(leaf(b"warm"), &p3), r);
    }
}
