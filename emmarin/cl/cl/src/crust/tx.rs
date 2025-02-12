use rand_core::{CryptoRngCore, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    crust::{
        balance::{Balance, BalanceWitness},
        iow::{BurnWitness, InputWitness, MintWitness, OutputWitness},
        NoteCommitment, Nullifier,
    },
    ds::mmr::{MMRProof, MMR},
    mantle::ZoneId,
    merkle, Digest, Hash,
};

pub const MAX_INPUTS: usize = 8;
pub const MAX_OUTPUTS: usize = 8;

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
    pub updates: Vec<LedgerUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxWitness {
    pub inputs: Vec<InputWitness>,
    pub outputs: Vec<(OutputWitness, Vec<u8>)>,
    pub data: Vec<u8>,
    pub balance_blinding: [u8; 16],
    pub mint_burn_blinding: [u8; 16],
    pub mints: Vec<MintWitness>,
    pub burns: Vec<BurnWitness>,
    pub frontier_paths: Vec<(MMR, MMRProof)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerUpdate {
    pub zone_id: ZoneId,
    pub frontier_nodes: Vec<MMR>,
    pub inputs: Vec<Nullifier>,
    pub outputs: Vec<NoteCommitment>,
}

pub struct LedgerUpdateWitness {
    pub zone_id: ZoneId,
    pub frontier_nodes: Vec<MMR>,
    pub inputs: Vec<Nullifier>,
    pub outputs: Vec<(NoteCommitment, Vec<u8>)>,
}

impl LedgerUpdateWitness {
    pub fn commit(self) -> (LedgerUpdate, [u8; 32]) {
        let input_root = merkle::root(&merkle::padded_leaves(
            &self.inputs.iter().map(|nf| nf.0).collect::<Vec<_>>(),
        ));
        let output_root = merkle::root(&merkle::padded_leaves(
            &self
                .outputs
                .iter()
                .map(|(cm, data)| {
                    cm.0.into_iter()
                        .chain(data.iter().cloned())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        ));
        let root = merkle::root(&merkle::padded_leaves(&[
            input_root,
            output_root,
            self.zone_id,
        ]));

        (
            LedgerUpdate {
                zone_id: self.zone_id,
                inputs: self.inputs,
                outputs: self.outputs.into_iter().map(|(cm, _)| cm).collect(),
                frontier_nodes: self.frontier_nodes,
            },
            root,
        )
    }
}

impl TxWitness {
    pub fn random(
        inputs: Vec<InputWitness>,
        outputs: Vec<(OutputWitness, Vec<u8>)>,
        burns: Vec<BurnWitness>,
        mints: Vec<MintWitness>,
        data: Vec<u8>,
        frontier_paths: Vec<(MMR, MMRProof)>,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        Self {
            inputs,
            outputs,
            data,
            burns,
            mints,
            balance_blinding: BalanceWitness::random_blinding(&mut rng),
            mint_burn_blinding: BalanceWitness::random_blinding(&mut rng), // TODO: type
            frontier_paths,
        }
    }

    pub fn balance(&self) -> BalanceWitness {
        BalanceWitness::from_tx(self, self.balance_blinding)
    }

    pub fn updates(&self) -> BTreeMap<ZoneId, LedgerUpdateWitness> {
        let mut updates = BTreeMap::new();
        assert_eq!(self.inputs.len(), self.frontier_paths.len());
        for (input, (mmr, path)) in self.inputs.iter().zip(&self.frontier_paths) {
            let entry = updates.entry(input.zone_id).or_insert(LedgerUpdateWitness {
                zone_id: input.zone_id,
                inputs: vec![],
                outputs: vec![],
                frontier_nodes: vec![],
            });
            entry.inputs.push(input.nullifier());
            assert!(mmr.verify_proof(&input.note_commitment().0, &path));
            entry.frontier_nodes.push(mmr.clone());
        }

        for (output, data) in &self.outputs {
            assert!(output.note.value > 0);
            updates
                .entry(output.zone_id)
                .or_insert(LedgerUpdateWitness {
                    zone_id: output.zone_id,
                    inputs: vec![],
                    outputs: vec![],
                    frontier_nodes: vec![],
                })
                .outputs
                .push((output.note_commitment(), data.clone())); // TODO: avoid clone
        }

        updates
    }

    pub fn mint_burn_root(&self) -> [u8; 32] {
        let mint_root = merkle::root(&merkle::padded_leaves(
            &self.mints.iter().map(|m| m.to_bytes()).collect::<Vec<_>>(),
        ));

        let burn_root = merkle::root(&merkle::padded_leaves(
            &self.burns.iter().map(|b| b.to_bytes()).collect::<Vec<_>>(),
        ));

        let mut hasher = Hash::new();
        hasher.update(&mint_root);
        hasher.update(&burn_root);
        hasher.update(&self.mint_burn_blinding);
        hasher.finalize().into()
    }

    pub fn commit(&self) -> Tx {
        let (updates, updates_roots): (Vec<_>, Vec<_>) =
            self.updates().into_iter().map(|(_, w)| w.commit()).unzip();

        let update_root = merkle::root(&merkle::padded_leaves(&updates_roots));
        let mint_burn_root = self.mint_burn_root();
        let data_root = merkle::leaf(&self.data);

        let root = TxRoot(merkle::root(&merkle::padded_leaves(&[
            update_root,
            mint_burn_root,
            data_root,
        ])));

        // TODO: fix
        let balance = BalanceWitness::from_tx(self, self.balance_blinding).commit();

        Tx {
            root,
            balance,
            updates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub updates: Vec<LedgerUpdate>,
    pub root: BundleRoot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleWitness {
    pub txs: Vec<Tx>,
    pub balance_witnesses: Vec<BalanceWitness>,
}

impl BundleWitness {
    pub fn commit(self) -> Bundle {
        assert_eq!(self.txs.len(), self.balance_witnesses.len());
        for (balance, witness) in self
            .txs
            .iter()
            .map(|tx| tx.balance)
            .zip(self.balance_witnesses.iter())
        {
            assert_eq!(balance, witness.commit());
        }
        assert!(BalanceWitness::combine(self.balance_witnesses, [0; 16]).is_zero());

        let root = BundleRoot(merkle::root(&merkle::padded_leaves(
            &self.txs.iter().map(|tx| tx.root.0).collect::<Vec<_>>(),
        )));

        let updates = self
            .txs
            .into_iter()
            .fold(BTreeMap::new(), |mut updates, tx| {
                for update in tx.updates {
                    let entry = updates.entry(update.zone_id).or_insert(LedgerUpdate {
                        zone_id: update.zone_id,
                        inputs: vec![],
                        outputs: vec![],
                        frontier_nodes: vec![],
                    });

                    entry.inputs.extend(update.inputs);
                    entry.outputs.extend(update.outputs);
                    entry.frontier_nodes.extend(update.frontier_nodes); // TODO: maybe merge?
                }

                updates
            })
            .into_iter()
            .map(|(_, update)| update)
            .collect::<Vec<_>>();
        Bundle { updates, root }
    }
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
