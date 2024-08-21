use serde::{Deserialize, Serialize};

use crate::{partial_tx::PartialTx, Balance, BalanceWitness};

/// The transaction bundle is a collection of partial transactions.
/// The goal in bundling transactions is to produce a set of partial transactions
/// that balance each other.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub partials: Vec<PartialTx>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleWitness {
    pub balance_blinding: BalanceWitness,
}

impl Bundle {
    pub fn balance(&self) -> Balance {
        Balance(self.partials.iter().map(|ptx| ptx.balance.0).sum())
    }

    pub fn is_balanced(&self, witness: BalanceWitness) -> bool {
        self.balance() == Balance::zero(witness)
    }
}

#[cfg(test)]
mod test {
    use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, Scalar};

    use crate::{
        input::InputWitness,
        note::{unit_point, NoteWitness},
        nullifier::NullifierSecret,
        output::OutputWitness,
        partial_tx::PartialTxWitness,
    };

    use super::*;

    #[test]
    fn test_bundle_balance() {
        let mut rng = rand::thread_rng();
        let (nmo, eth, crv) = (unit_point("NMO"), unit_point("ETH"), unit_point("CRV"));

        let nf_a = NullifierSecret::random(&mut rng);
        let nf_b = NullifierSecret::random(&mut rng);
        let nf_c = NullifierSecret::random(&mut rng);

        let nmo_10_utxo =
            OutputWitness::random(NoteWitness::basic(10, nmo), nf_a.commit(), &mut rng);
        let nmo_10_in = InputWitness::from_output(nmo_10_utxo, nf_a);

        let eth_23_utxo =
            OutputWitness::random(NoteWitness::basic(23, eth), nf_b.commit(), &mut rng);
        let eth_23_in = InputWitness::from_output(eth_23_utxo, nf_b);

        let crv_4840_out =
            OutputWitness::random(NoteWitness::basic(4840, crv), nf_c.commit(), &mut rng);

        let ptx_unbalanced = PartialTxWitness {
            inputs: vec![nmo_10_in, eth_23_in],
            outputs: vec![crv_4840_out],
            balance_blinding: BalanceWitness::random(&mut rng),
        };

        let bundle_witness = BundleWitness {
            balance_blinding: ptx_unbalanced.balance_blinding,
        };

        let mut bundle = Bundle {
            partials: vec![ptx_unbalanced.commit()],
        };

        let crv_4840_out_bal = crv_4840_out.note.unit * Scalar::from(crv_4840_out.note.value);
        let nmo_10_in_bal = nmo_10_in.note.unit * Scalar::from(nmo_10_in.note.value);
        let eth_23_in_bal = eth_23_in.note.unit * Scalar::from(eth_23_in.note.value);
        let unbalance_blinding = RISTRETTO_BASEPOINT_POINT * ptx_unbalanced.balance_blinding.0;
        assert!(!bundle.is_balanced(bundle_witness.balance_blinding));
        assert_eq!(
            bundle.balance().0,
            crv_4840_out_bal - (nmo_10_in_bal + eth_23_in_bal) + unbalance_blinding
        );

        let crv_4840_in = InputWitness::from_output(crv_4840_out, nf_c);
        let nmo_10_out = OutputWitness::random(
            NoteWitness::basic(10, nmo),
            NullifierSecret::random(&mut rng).commit(), // transferring to a random owner
            &mut rng,
        );
        let eth_23_out = OutputWitness::random(
            NoteWitness::basic(23, eth),
            NullifierSecret::random(&mut rng).commit(), // transferring to a random owner
            &mut rng,
        );

        let ptx_solved = PartialTxWitness {
            inputs: vec![crv_4840_in],
            outputs: vec![nmo_10_out, eth_23_out],
            balance_blinding: BalanceWitness::random(&mut rng),
        };

        bundle.partials.push(ptx_solved.commit());

        let witness = BundleWitness {
            balance_blinding: BalanceWitness::new(
                ptx_unbalanced.balance_blinding.0 + ptx_solved.balance_blinding.0,
            ),
        };

        assert!(bundle.is_balanced(witness.balance_blinding));
    }
}
