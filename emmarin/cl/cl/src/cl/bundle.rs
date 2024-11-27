use serde::{Deserialize, Serialize};

use crate::{cl::partial_tx::PartialTx, zone_layer::notes::ZoneId};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// The transaction bundle is a collection of partial transactions.
/// The goal in bundling transactions is to produce a set of partial transactions
/// that balance each other.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BundleId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bundle {
    pub partials: Vec<PartialTx>,
}

impl Bundle {
    pub fn zones(&self) -> HashSet<ZoneId> {
        self.partials
            .iter()
            .flat_map(|ptx| {
                ptx.inputs
                    .iter()
                    .map(|i| i.zone_id)
                    .chain(ptx.outputs.iter().map(|o| o.zone_id))
            })
            .collect()
    }

    ///
    pub fn id(&self) -> BundleId {
        // TODO: change to merkle root
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_CL_BUNDLE_ID");
        for ptx in &self.partials {
            hasher.update(&ptx.root().0);
        }

        BundleId(hasher.finalize().into())
    }
}

#[cfg(test)]
mod test {
    use crate::cl::{
        balance::{BalanceWitness, UnitBalance},
        input::InputWitness,
        note::{derive_unit, NoteWitness},
        nullifier::NullifierSecret,
        output::OutputWitness,
        partial_tx::PartialTxWitness,
    };

    #[test]
    fn test_bundle_balance() {
        let mut rng = rand::thread_rng();
        let zone_id = [0; 32];
        let (nmo, eth, crv) = (derive_unit("NMO"), derive_unit("ETH"), derive_unit("CRV"));

        let nf_a = NullifierSecret::random(&mut rng);
        let nf_b = NullifierSecret::random(&mut rng);
        let nf_c = NullifierSecret::random(&mut rng);

        let nmo_10_utxo = OutputWitness::new(
            NoteWitness::basic(10, nmo, &mut rng),
            nf_a.commit(),
            zone_id,
        );
        let nmo_10_in = InputWitness::from_output(nmo_10_utxo, nf_a);

        let eth_23_utxo = OutputWitness::new(
            NoteWitness::basic(23, eth, &mut rng),
            nf_b.commit(),
            zone_id,
        );
        let eth_23_in = InputWitness::from_output(eth_23_utxo, nf_b);

        let crv_4840_out = OutputWitness::new(
            NoteWitness::basic(4840, crv, &mut rng),
            nf_c.commit(),
            zone_id,
        );

        let ptx_unbalanced = PartialTxWitness {
            inputs: vec![nmo_10_in, eth_23_in],
            outputs: vec![crv_4840_out],
            balance_blinding: BalanceWitness::random_blinding(&mut rng),
        };

        assert!(!ptx_unbalanced.balance().is_zero());
        assert_eq!(
            ptx_unbalanced.balance().balances,
            vec![
                UnitBalance {
                    unit: nmo,
                    pos: 0,
                    neg: 10
                },
                UnitBalance {
                    unit: eth,
                    pos: 0,
                    neg: 23
                },
                UnitBalance {
                    unit: crv,
                    pos: 4840,
                    neg: 0
                },
            ]
        );

        let crv_4840_in = InputWitness::from_output(crv_4840_out, nf_c);
        let nmo_10_out = OutputWitness::new(
            NoteWitness::basic(10, nmo, &mut rng),
            NullifierSecret::random(&mut rng).commit(), // transferring to a random owner
            zone_id,
        );
        let eth_23_out = OutputWitness::new(
            NoteWitness::basic(23, eth, &mut rng),
            NullifierSecret::random(&mut rng).commit(), // transferring to a random owner
            zone_id,
        );

        let ptx_solved = PartialTxWitness {
            inputs: vec![crv_4840_in],
            outputs: vec![nmo_10_out, eth_23_out],
            balance_blinding: BalanceWitness::random_blinding(&mut rng),
        };

        let bundle_balance =
            BalanceWitness::combine([ptx_unbalanced.balance(), ptx_solved.balance()], [0; 16]);

        assert!(bundle_balance.is_zero());
        assert_eq!(bundle_balance.balances, vec![]);
    }
}
