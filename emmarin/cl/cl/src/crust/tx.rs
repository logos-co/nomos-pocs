use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    crust::{
        Balance, BurnWitness, InputWitness, MintWitness, NoteCommitment, Nullifier, OutputWitness,
        Unit,
    },
    ds::{
        merkle,
        mmr::{MMRProof, Root, MMR},
    },
    mantle::ZoneId,
};

/// An identifier of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TxRoot(pub [u8; 32]);

/// An identifier of a bundle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BundleRoot(pub [u8; 32]);

impl From<[u8; 32]> for TxRoot {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; 32]> for BundleRoot {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl TxRoot {
    pub fn random(mut rng: impl RngCore) -> Self {
        let mut sk = [0u8; 32];
        rng.fill_bytes(&mut sk);
        Self(sk)
    }

    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tx {
    pub root: TxRoot,
    pub balance: Balance,
    pub updates: BTreeMap<ZoneId, LedgerUpdate>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TxWitness {
    pub inputs: Vec<InputWitness>,
    pub outputs: Vec<(OutputWitness, Vec<u8>)>,
    pub data: Vec<u8>,
    pub mints: Vec<MintWitness>,
    pub burns: Vec<BurnWitness>,
    pub frontier_paths: Vec<(MMR, MMRProof)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LedgerUpdate {
    pub frontier_nodes: Vec<Root>,
    pub inputs: Vec<Nullifier>,
    pub outputs: Vec<(NoteCommitment, Vec<u8>)>,
}

impl LedgerUpdate {
    pub fn root(&self, zone_id: ZoneId) -> [u8; 32] {
        let input_root = merkle::root(&merkle::padded_leaves(&self.inputs));
        let output_root = merkle::root(&merkle::padded_leaves(self.outputs.iter().map(
            |(cm, data)| {
                cm.0.into_iter()
                    .chain(data.iter().cloned())
                    .collect::<Vec<_>>()
            },
        )));
        merkle::root(&merkle::padded_leaves([zone_id, input_root, output_root]))
    }

    pub fn add_input(&mut self, nf: Nullifier, mmr: MMR) -> &mut Self {
        self.inputs.push(nf);
        self.frontier_nodes.extend(mmr.roots);
        self.frontier_nodes.sort();
        self.frontier_nodes.dedup();
        self
    }

    pub fn add_output(&mut self, cm: NoteCommitment, data: Vec<u8>) -> &mut Self {
        self.outputs.push((cm, data));
        self
    }

    pub fn has_input(&self, nf: &Nullifier) -> bool {
        self.inputs.contains(nf)
    }

    pub fn has_output(&self, cm: &NoteCommitment) -> bool {
        self.outputs.iter().any(|(out_cm, _data)| out_cm == cm)
    }
}

impl TxWitness {
    pub fn add_input(mut self, input: InputWitness, input_cm_proof: (MMR, MMRProof)) -> Self {
        self.inputs.push(input);
        self.frontier_paths.push(input_cm_proof);
        self
    }

    pub fn add_output(mut self, output: OutputWitness, data: Vec<u8>) -> Self {
        self.outputs.push((output, data));
        self
    }

    pub fn compute_updates(&self, inputs: &[InputDerivedFields]) -> BTreeMap<ZoneId, LedgerUpdate> {
        let mut updates: BTreeMap<ZoneId, LedgerUpdate> = Default::default();

        assert_eq!(self.inputs.len(), self.frontier_paths.len());
        for (input, (mmr, path)) in inputs.iter().zip(&self.frontier_paths) {
            let entry = updates
                .entry(input.zone_id)
                .or_default()
                .add_input(input.nf, mmr.clone());

            assert!(mmr.verify_proof(&input.cm.0, path));
            // ensure a single MMR per zone per tx
            assert_eq!(&mmr.roots, &entry.frontier_nodes);
        }

        for (output, data) in &self.outputs {
            assert!(output.value > 0);
            updates
                .entry(output.zone_id)
                .or_default()
                .add_output(output.note_commitment(), data.clone()); // TODO: avoid clone
        }

        updates
    }

    pub fn mint_amounts(&self) -> Vec<MintAmount> {
        self.mints
            .iter()
            .map(|MintWitness { unit, amount, salt }| MintAmount {
                unit: unit.unit(),
                amount: *amount,
                salt: *salt,
            })
            .collect()
    }

    pub fn burn_amounts(&self) -> Vec<BurnAmount> {
        self.burns
            .iter()
            .map(|BurnWitness { unit, amount, salt }| BurnAmount {
                unit: unit.unit(),
                amount: *amount,
                salt: *salt,
            })
            .collect()
    }

    pub fn inputs_derived_fields(&self) -> Vec<InputDerivedFields> {
        self.inputs
            .iter()
            .map(|input| InputDerivedFields {
                nf: input.nullifier(),
                cm: input.note_commitment(),
                zone_id: input.zone_id,
            })
            .collect()
    }

    pub fn mint_burn_root(mints: &[MintAmount], burns: &[BurnAmount]) -> [u8; 32] {
        let mint_root = merkle::root(&merkle::padded_leaves(mints.iter().map(|m| m.to_bytes())));
        let burn_root = merkle::root(&merkle::padded_leaves(burns.iter().map(|b| b.to_bytes())));
        merkle::node(mint_root, burn_root)
    }

    fn io_balance(&self) -> Balance {
        let mut balance = Balance::zero();
        for input in &self.inputs {
            balance.insert_positive(input.unit_witness.unit(), input.value);
        }
        for (output, _) in &self.outputs {
            balance.insert_negative(output.unit, output.value);
        }
        balance
    }

    pub fn root(&self, update_root: [u8; 32], mint_burn_root: [u8; 32]) -> TxRoot {
        let data_root = merkle::leaf(&self.data);
        let root = merkle::root(&merkle::padded_leaves([
            update_root,
            mint_burn_root,
            data_root,
        ]));
        TxRoot(root)
    }

    pub fn balance(&self, mints: &[MintAmount], burns: &[BurnAmount]) -> Balance {
        let mut mint_burn_balance = Balance::zero();
        for MintAmount { unit, amount, .. } in mints {
            mint_burn_balance.insert_positive(*unit, *amount);
        }
        for BurnAmount { unit, amount, .. } in burns {
            mint_burn_balance.insert_negative(*unit, *amount);
        }
        Balance::combine(&[mint_burn_balance, self.io_balance()])
    }

    // inputs, mints and burns are provided as a separate argument to allow code reuse
    // with the proof without having to recompute them
    pub fn commit(
        &self,
        mints: &[MintAmount],
        burns: &[BurnAmount],
        inputs: &[InputDerivedFields],
    ) -> Tx {
        let mint_burn_root = Self::mint_burn_root(mints, burns);

        let updates = self.compute_updates(inputs);
        let update_root = merkle::root(&merkle::padded_leaves(
            updates
                .iter()
                .map(|(zone_id, update)| update.root(*zone_id)),
        ));
        let root = self.root(update_root, mint_burn_root);
        let balance = self.balance(mints, burns);

        Tx {
            root,
            balance,
            updates,
            data: self.data.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub updates: BTreeMap<ZoneId, LedgerUpdate>,
    pub root: BundleRoot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleWitness {
    pub txs: Vec<Tx>,
}

impl BundleWitness {
    pub fn root(&self) -> BundleRoot {
        BundleRoot(merkle::root(&merkle::padded_leaves(
            self.txs.iter().map(|tx| tx.root.0),
        )))
    }

    pub fn commit(self) -> Bundle {
        assert!(Balance::combine(self.txs.iter().map(|tx| &tx.balance)).is_zero());

        let root = self.root();

        let mut updates = self
            .txs
            .into_iter()
            .fold(BTreeMap::new(), |mut updates, tx| {
                for (zone_id, update) in tx.updates {
                    let entry: &mut LedgerUpdate = updates.entry(zone_id).or_default();
                    entry.inputs.extend(update.inputs);
                    entry.outputs.extend(update.outputs);
                    entry.frontier_nodes.extend(update.frontier_nodes); // TODO: maybe merge?
                }

                updates
            });

        // de-dup frontier nodes
        updates.iter_mut().for_each(|(_, update)| {
            update.frontier_nodes.sort();
            update.frontier_nodes.dedup();
        });

        Bundle { updates, root }
    }
}

// ----- Helper structs -----
// To validate the unit covenants we need the tx root plus some additional information that is computed to
// calculate the tx root. To avoid recomputation we store this information in the following structs.

pub struct MintAmount {
    pub unit: Unit,
    pub amount: u64,
    pub salt: [u8; 16],
}

impl MintAmount {
    fn to_bytes(&self) -> [u8; 56] {
        let mut bytes = [0; 56];
        bytes[..32].copy_from_slice(&self.unit);
        bytes[32..40].copy_from_slice(&self.amount.to_le_bytes());
        bytes[40..].copy_from_slice(&self.salt);
        bytes
    }
}

pub struct BurnAmount {
    pub unit: Unit,
    pub amount: u64,
    pub salt: [u8; 16],
}

impl BurnAmount {
    fn to_bytes(&self) -> [u8; 56] {
        let mut bytes = [0; 56];
        bytes[..32].copy_from_slice(&self.unit);
        bytes[32..40].copy_from_slice(&self.amount.to_le_bytes());
        bytes[40..].copy_from_slice(&self.salt);
        bytes
    }
}

pub struct InputDerivedFields {
    pub nf: Nullifier,
    pub cm: NoteCommitment,
    pub zone_id: ZoneId,
}

#[cfg(test)]
mod test {

    // use crate::cl::{
    //     balance::UnitBalance,
    //     note::{derive_unit, NoteWitness},
    //     nullifier::NullifierSecret,
    // };

    // use super::*;

    // #[test]
    // fn test_partial_tx_balance() {
    //     let (nmo, eth, crv) = (derive_unit("NMO"), derive_unit("ETH"), derive_unit("CRV"));
    //     let mut rng = rand::thread_rng();

    //     let nf_a = NullifierSecret::random(&mut rng);
    //     let nf_b = NullifierSecret::random(&mut rng);
    //     let nf_c = NullifierSecret::random(&mut rng);

    //     let nmo_10_utxo = OutputWitness::new(NoteWitness::basic(10, nmo, &mut rng), nf_a.commit());
    //     let nmo_10 = InputWitness::from_output(nmo_10_utxo, nf_a);

    //     let eth_23_utxo = OutputWitness::new(NoteWitness::basic(23, eth, &mut rng), nf_b.commit());
    //     let eth_23 = InputWitness::from_output(eth_23_utxo, nf_b);

    //     let crv_4840 = OutputWitness::new(NoteWitness::basic(4840, crv, &mut rng), nf_c.commit());

    //     let ptx_witness = TxWitness {
    //         inputs: vec![nmo_10, eth_23],
    //         outputs: vec![crv_4840],
    //         balance_blinding: BalanceWitness::random_blinding(&mut rng),
    //     };

    //     let ptx = ptx_witness.commit();

    //     assert_eq!(
    //         ptx.balance,
    //         BalanceWitness {
    //             balances: vec![
    //                 UnitBalance {
    //                     unit: nmo,
    //                     pos: 0,
    //                     neg: 10
    //                 },
    //                 UnitBalance {
    //                     unit: eth,
    //                     pos: 0,
    //                     neg: 23
    //                 },
    //                 UnitBalance {
    //                     unit: crv,
    //                     pos: 4840,
    //                     neg: 0
    //                 },
    //             ],
    //             blinding: ptx_witness.balance_blinding
    //         }
    //         .commit()
    //     );
    // }
}
