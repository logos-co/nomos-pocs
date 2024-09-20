use itertools::Itertools;
use ark_bls12_381::Fr as BlsFr;

use super::CpuBackend;
use crate::core::fields::m31::BaseField;
use crate::core::vcs::ops::{MerkleHasher, MerkleOps};
use crate::core::vcs::poseidon_bls_merkle::PoseidonBLSMerkleHasher;

impl MerkleOps<PoseidonBLSMerkleHasher> for CpuBackend {
    fn commit_on_layer(
        log_size: u32,
        prev_layer: Option<&Vec<BlsFr>>,
        columns: &[&Vec<BaseField>],
    ) -> Vec<BlsFr> {
        (0..(1 << log_size))
            .map(|i| {
                PoseidonBLSMerkleHasher::hash_node(
                    prev_layer.map(|prev_layer| (prev_layer[2 * i], prev_layer[2 * i + 1])),
                    &columns.iter().map(|column| column[i]).collect_vec(),
                )
            })
            .collect()
    }
}
