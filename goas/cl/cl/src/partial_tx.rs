use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::Scalar;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::balance::{Balance, BalanceWitness};
use crate::input::{Input, InputWitness};
use crate::merkle;
use crate::output::{Output, OutputWitness};

const MAX_INPUTS: usize = 8;
const MAX_OUTPUTS: usize = 8;

/// The partial transaction commitment couples an input to a partial transaction.
/// Prevents partial tx unbundling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PtxRoot(pub [u8; 32]);

impl From<[u8; 32]> for PtxRoot {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl PtxRoot {
    pub fn random(mut rng: impl RngCore) -> Self {
        let mut sk = [0u8; 32];
        rng.fill_bytes(&mut sk);
        Self(sk)
    }

    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialTx {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone)]
pub struct PartialTxWitness {
    pub inputs: Vec<InputWitness>,
    pub outputs: Vec<OutputWitness>,
}

impl PartialTxWitness {
    pub fn commit(&self) -> PartialTx {
        PartialTx {
            inputs: Vec::from_iter(self.inputs.iter().map(InputWitness::commit)),
            outputs: Vec::from_iter(self.outputs.iter().map(OutputWitness::commit)),
        }
    }

    pub fn balance_delta(&self) -> i128 {
        let in_sum: i128 = self.inputs.iter().map(|i| i.note.value as i128).sum();
        let out_sum: i128 = self.outputs.iter().map(|o| o.note.value as i128).sum();

        out_sum - in_sum
    }

    pub fn balance_blinding(&self) -> BalanceWitness {
        let in_sum: Scalar = self.inputs.iter().map(|i| i.balance_blinding.0).sum();
        let out_sum: Scalar = self.outputs.iter().map(|o| o.balance_blinding.0).sum();

        BalanceWitness(out_sum - in_sum)
    }
}

impl PartialTx {
    pub fn input_root(&self) -> [u8; 32] {
        let input_bytes =
            Vec::from_iter(self.inputs.iter().map(Input::to_bytes).map(Vec::from_iter));
        let input_merkle_leaves = merkle::padded_leaves(&input_bytes);
        merkle::root::<MAX_INPUTS>(input_merkle_leaves)
    }

    pub fn output_root(&self) -> [u8; 32] {
        let output_bytes = Vec::from_iter(
            self.outputs
                .iter()
                .map(Output::to_bytes)
                .map(Vec::from_iter),
        );
        let output_merkle_leaves = merkle::padded_leaves(&output_bytes);
        merkle::root::<MAX_OUTPUTS>(output_merkle_leaves)
    }

    pub fn input_merkle_path(&self, idx: usize) -> Vec<merkle::PathNode> {
        let input_bytes =
            Vec::from_iter(self.inputs.iter().map(Input::to_bytes).map(Vec::from_iter));
        let input_merkle_leaves = merkle::padded_leaves::<MAX_INPUTS>(&input_bytes);
        merkle::path(input_merkle_leaves, idx)
    }

    pub fn output_merkle_path(&self, idx: usize) -> Vec<merkle::PathNode> {
        let output_bytes = Vec::from_iter(
            self.outputs
                .iter()
                .map(Output::to_bytes)
                .map(Vec::from_iter),
        );
        let output_merkle_leaves = merkle::padded_leaves::<MAX_OUTPUTS>(&output_bytes);
        merkle::path(output_merkle_leaves, idx)
    }

    pub fn root(&self) -> PtxRoot {
        let input_root = self.input_root();
        let output_root = self.output_root();
        let root = merkle::node(input_root, output_root);
        PtxRoot(root)
    }

    pub fn balance(&self) -> Balance {
        let in_sum: RistrettoPoint = self.inputs.iter().map(|i| i.balance.0).sum();
        let out_sum: RistrettoPoint = self.outputs.iter().map(|o| o.balance.0).sum();

        Balance(out_sum - in_sum)
    }
}

#[cfg(test)]
mod test {

    use crate::{note::NoteWitness, nullifier::NullifierSecret};

    use super::*;

    #[test]
    fn test_partial_tx_balance() {
        let mut rng = rand::thread_rng();

        let nf_a = NullifierSecret::random(&mut rng);
        let nf_b = NullifierSecret::random(&mut rng);
        let nf_c = NullifierSecret::random(&mut rng);

        let nmo_10_utxo =
            OutputWitness::random(NoteWitness::basic(10, "NMO"), nf_a.commit(), &mut rng);
        let nmo_10 = InputWitness::random(nmo_10_utxo, nf_a, &mut rng);

        let eth_23_utxo =
            OutputWitness::random(NoteWitness::basic(23, "ETH"), nf_b.commit(), &mut rng);
        let eth_23 = InputWitness::random(eth_23_utxo, nf_b, &mut rng);

        let crv_4840 =
            OutputWitness::random(NoteWitness::basic(4840, "CRV"), nf_c.commit(), &mut rng);

        let ptx_witness = PartialTxWitness {
            inputs: vec![nmo_10, eth_23],
            outputs: vec![crv_4840],
        };

        let ptx = ptx_witness.commit();

        assert_eq!(
            ptx.balance().0,
            crv_4840.commit().balance.0 - (nmo_10.commit().balance.0 + eth_23.commit().balance.0)
        );
    }
}
