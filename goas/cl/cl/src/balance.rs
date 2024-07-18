use curve25519_dalek::{ristretto::RistrettoPoint, traits::VartimeMultiscalarMul, Scalar};
use lazy_static::lazy_static;
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};

use crate::NoteWitness;

lazy_static! {
    // Precompute of ``
    static ref PEDERSON_COMMITMENT_BLINDING_POINT: RistrettoPoint = crate::crypto::hash_to_curve(b"NOMOS_CL_PEDERSON_COMMITMENT_BLINDING");
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Balance(pub RistrettoPoint);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct BalanceWitness(pub Scalar);

impl Balance {
    /// A commitment to zero, blinded by the provided balance witness
    pub fn zero(blinding: BalanceWitness) -> Self {
        Self(balance(
            0,
            curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT,
            blinding.0,
        ))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }
}

impl BalanceWitness {
    pub fn new(blinding: Scalar) -> Self {
        Self(blinding)
    }

    pub fn random(mut rng: impl CryptoRngCore) -> Self {
        Self::new(Scalar::random(&mut rng))
    }

    pub fn commit(&self, note: &NoteWitness) -> Balance {
        Balance(balance(note.value, note.unit, self.0))
    }
}

pub fn balance(value: u64, unit: RistrettoPoint, blinding: Scalar) -> RistrettoPoint {
    let value_scalar = Scalar::from(value);
    // can vartime leak the number of cycles through the stark proof?
    RistrettoPoint::vartime_multiscalar_mul(
        &[value_scalar, blinding],
        &[unit, *PEDERSON_COMMITMENT_BLINDING_POINT],
    )
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_pederson_blinding_point_pre_compute() {
        // use k256::elliptic_curve::group::GroupEncoding;
        // println!("{:?}", <[u8;33]>::from((*PEDERSON_COMMITMENT_BLINDING_POINT).to_bytes()));

        assert_eq!(
            *PEDERSON_COMMITMENT_BLINDING_POINT,
            crate::crypto::hash_to_curve(b"NOMOS_CL_PEDERSON_COMMITMENT_BLINDING")
        );
    }

    #[test]
    fn test_balance_zero_unitless() {
        // Zero is the same across all units
        let mut rng = rand::thread_rng();
        let b = BalanceWitness::random(&mut rng);
        assert_eq!(
            b.commit(&NoteWitness::basic(0, "NMO")),
            b.commit(&NoteWitness::basic(0, "ETH")),
        );
    }

    #[test]
    fn test_balance_blinding() {
        // balances are blinded
        let r_a = Scalar::from(12u32);
        let r_b = Scalar::from(8u32);
        let bal_a = BalanceWitness::new(r_a);
        let bal_b = BalanceWitness::new(r_b);

        let note = NoteWitness::basic(10, "NMO");

        let a = bal_a.commit(&note);
        let b = bal_b.commit(&note);

        assert_ne!(a, b);

        let diff_note = NoteWitness::basic(0, "NMO");
        assert_eq!(
            a.0 - b.0,
            BalanceWitness::new(r_a - r_b).commit(&diff_note).0
        );
    }

    #[test]
    fn test_balance_units() {
        // Unit's differentiate between values.
        let b = BalanceWitness::new(Scalar::from(1337u32));

        let nmo = NoteWitness::basic(10, "NMO");
        let eth = NoteWitness::basic(10, "ETH");
        assert_ne!(b.commit(&nmo), b.commit(&eth));
    }

    #[test]
    fn test_balance_homomorphism() {
        let mut rng = rand::thread_rng();
        let b1 = BalanceWitness::random(&mut rng);
        let b2 = BalanceWitness::random(&mut rng);
        let b_zero = BalanceWitness::new(Scalar::ZERO);

        let ten = NoteWitness::basic(10, "NMO");
        let eight = NoteWitness::basic(8, "NMO");
        let two = NoteWitness::basic(2, "NMO");
        let zero = NoteWitness::basic(0, "NMO");

        // Values of same unit are homomorphic
        assert_eq!(
            (b1.commit(&ten).0 - b1.commit(&eight).0),
            b_zero.commit(&two).0
        );

        // Blinding factors are also homomorphic.
        assert_eq!(
            b1.commit(&ten).0 - b2.commit(&ten).0,
            BalanceWitness::new(b1.0 - b2.0).commit(&zero).0
        );
    }
}
