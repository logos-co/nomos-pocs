use itertools::Itertools;
use ark_bls12_381::Fr as BlsFr;

use super::SimdBackend;
use crate::core::backend::{Col, Column, ColumnOps};
use crate::core::fields::m31::BaseField;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::vcs::ops::MerkleHasher;
use crate::core::vcs::ops::MerkleOps;
use crate::core::vcs::poseidon_bls_merkle::PoseidonBLSMerkleHasher;

impl ColumnOps<BlsFr> for SimdBackend {
    type Column = Vec<BlsFr>;

    fn bit_reverse_column(_column: &mut Self::Column) {
        unimplemented!()
    }
}

impl MerkleOps<PoseidonBLSMerkleHasher> for SimdBackend {
    // TODO(ShaharS): replace with SIMD implementation.
    fn commit_on_layer(
        log_size: u32,
        prev_layer: Option<&Vec<BlsFr>>,
        columns: &[&Col<Self, BaseField>],
    ) -> Vec<BlsFr> {
        (0..(1 << log_size))
            .map(|i| {
                PoseidonBLSMerkleHasher::hash_node(
                    prev_layer.map(|prev_layer| (prev_layer[2 * i], prev_layer[2 * i + 1])),
                    &columns.iter().map(|column| column.at(i)).collect_vec(),
                )
            })
            .collect()
    }
}
