use serde::{Deserialize, Serialize};

use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, ristretto::RistrettoPoint};

use crate::{partial_tx::PartialTx, BalanceWitness};

/// The transaction bundle is a collection of partial transactions.
/// The goal in bundling transactions is to produce a set of partial transactions
/// that balance each other.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub partials: Vec<PartialTx>,
}

#[derive(Debug, Clone)]
pub struct BundleWitness {
    pub balance: BalanceWitness,
}

impl Bundle {
    pub fn balance(&self) -> RistrettoPoint {
        self.partials.iter().map(|ptx| ptx.balance()).sum()
    }

    pub fn is_balanced(&self, witness: BalanceWitness) -> bool {
        self.balance() == crate::balance::balance(0, RISTRETTO_BASEPOINT_POINT, witness.0)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        crypto::hash_to_curve, input::InputWitness, note::NoteWitness, nullifier::NullifierSecret,
        output::OutputWitness, partial_tx::PartialTxWitness,
    };

    use super::*;

    #[test]
    fn test_bundle_balance() {
        let mut rng = rand::thread_rng();

        let nf_a = NullifierSecret::random(&mut rng);
        let nf_b = NullifierSecret::random(&mut rng);
        let nf_c = NullifierSecret::random(&mut rng);

        let nmo_10_utxo =
            OutputWitness::random(NoteWitness::basic(10, "NMO"), nf_a.commit(), &mut rng);
        let nmo_10_in = InputWitness::random(nmo_10_utxo, nf_a, &mut rng);

        let eth_23_utxo =
            OutputWitness::random(NoteWitness::basic(23, "ETH"), nf_b.commit(), &mut rng);
        let eth_23_in = InputWitness::random(eth_23_utxo, nf_b, &mut rng);

        let crv_4840_out =
            OutputWitness::random(NoteWitness::basic(4840, "CRV"), nf_c.commit(), &mut rng);

        let ptx_unbalanced = PartialTxWitness {
            inputs: vec![nmo_10_in.clone(), eth_23_in.clone()],
            outputs: vec![crv_4840_out.clone()],
        };

        let bundle_witness = BundleWitness {
            balance: BalanceWitness::new(
                crv_4840_out.balance_blinding.0
                    - nmo_10_in.balance_blinding.0
                    - eth_23_in.balance_blinding.0,
            ),
        };

        let mut bundle = Bundle {
            partials: vec![PartialTx::from_witness(ptx_unbalanced)],
        };

        assert!(!bundle.is_balanced(bundle_witness.balance));
        assert_eq!(
            bundle.balance(),
            crate::balance::balance(4840, hash_to_curve(b"CRV"), crv_4840_out.balance_blinding.0)
                - (crate::balance::balance(
                    10,
                    hash_to_curve(b"NMO"),
                    nmo_10_in.balance_blinding.0
                ) + crate::balance::balance(
                    23,
                    hash_to_curve(b"ETH"),
                    eth_23_in.balance_blinding.0
                ))
        );

        let crv_4840_in = InputWitness::random(crv_4840_out, nf_c, &mut rng);
        let nmo_10_out = OutputWitness::random(
            NoteWitness::basic(10, "NMO"),
            NullifierSecret::random(&mut rng).commit(), // transferring to a random owner
            &mut rng,
        );
        let eth_23_out = OutputWitness::random(
            NoteWitness::basic(23, "ETH"),
            NullifierSecret::random(&mut rng).commit(), // transferring to a random owner
            &mut rng,
        );

        bundle
            .partials
            .push(PartialTx::from_witness(PartialTxWitness {
                inputs: vec![crv_4840_in.clone()],
                outputs: vec![nmo_10_out.clone(), eth_23_out.clone()],
            }));

        let witness = BundleWitness {
            balance: BalanceWitness::new(
                -nmo_10_in.balance_blinding.0 - eth_23_in.balance_blinding.0
                    + crv_4840_out.balance_blinding.0
                    - crv_4840_in.balance_blinding.0
                    + nmo_10_out.balance_blinding.0
                    + eth_23_out.balance_blinding.0,
            ),
        };

        assert_eq!(
            bundle.balance(),
            crate::balance::balance(
                0,
                curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT,
                witness.balance.0
            )
        );

        assert!(bundle.is_balanced(witness.balance));
    }
}
