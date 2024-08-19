use curve25519_dalek::{ristretto::RistrettoPoint, Scalar};
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};

use crate::NoteWitness;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Balance(pub RistrettoPoint);

pub type Value = u64;
pub type Unit = RistrettoPoint;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct BalanceWitness(pub Scalar);

impl Balance {
    /// A commitment to zero, blinded by the provided balance witness
    pub fn zero(blinding: BalanceWitness) -> Self {
        // Since, balance commitments are `value * UnitPoint + blinding * H`, when value=0, the commmitment is unitless.
        // So we use the generator point as a stand in for the unit point.
        //
        // TAI: we can optimize this further from `0*G + r*H` to just `r*H` to save a point scalar mult + point addition.
        let unit = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
        Self(balance(0, unit, blinding.0))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }
}

impl BalanceWitness {
    pub fn new(blinding: Scalar) -> Self {
        Self(blinding)
    }

    pub fn unblinded() -> Self {
        Self::new(Scalar::ZERO)
    }

    pub fn random(mut rng: impl CryptoRngCore) -> Self {
        Self::new(Scalar::random(&mut rng))
    }

    pub fn commit(&self, note: &NoteWitness) -> Balance {
        Balance(balance(note.value, note.unit, self.0))
    }
}

pub fn balance(value: u64, unit: Unit, blinding: Scalar) -> Unit {
    let value_scalar = Scalar::from(value);
    // can vartime leak the number of cycles through the stark proof?
    RistrettoPoint::vartime_double_scalar_mul_basepoint(&value_scalar, &unit, &blinding)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::note::unit_point;

    #[test]
    fn test_balance_zero_unitless() {
        // Zero is the same across all units
        let (nmo, eth) = (unit_point("NMO"), unit_point("ETH"));

        let mut rng = rand::thread_rng();
        let b = BalanceWitness::random(&mut rng);
        assert_eq!(
            b.commit(&NoteWitness::basic(0, nmo)),
            b.commit(&NoteWitness::basic(0, eth)),
        );
    }

    #[test]
    fn test_balance_blinding() {
        // balances are blinded
        let nmo = unit_point("NMO");

        let r_a = Scalar::from(12u32);
        let r_b = Scalar::from(8u32);
        let bal_a = BalanceWitness::new(r_a);
        let bal_b = BalanceWitness::new(r_b);

        let note = NoteWitness::basic(10, nmo);

        let a = bal_a.commit(&note);
        let b = bal_b.commit(&note);

        assert_ne!(a, b);

        let diff_note = NoteWitness::basic(0, nmo);
        assert_eq!(
            a.0 - b.0,
            BalanceWitness::new(r_a - r_b).commit(&diff_note).0
        );
    }

    #[test]
    fn test_balance_units() {
        // Unit's differentiate between values.
        let (nmo, eth) = (unit_point("NMO"), unit_point("ETH"));

        let b = BalanceWitness::new(Scalar::from(1337u32));

        let nmo = NoteWitness::basic(10, nmo);
        let eth = NoteWitness::basic(10, eth);
        assert_ne!(b.commit(&nmo), b.commit(&eth));
    }

    #[test]
    fn test_balance_homomorphism() {
        let nmo = unit_point("NMO");

        let mut rng = rand::thread_rng();
        let b1 = BalanceWitness::random(&mut rng);
        let b2 = BalanceWitness::random(&mut rng);
        let b_zero = BalanceWitness::new(Scalar::ZERO);

        let ten = NoteWitness::basic(10, nmo);
        let eight = NoteWitness::basic(8, nmo);
        let two = NoteWitness::basic(2, nmo);
        let zero = NoteWitness::basic(0, nmo);

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
