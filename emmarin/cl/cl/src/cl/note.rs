use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cl::{balance::Unit, nullifier::NullifierCommitment};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Constraint(pub [u8; 32]);

impl Constraint {
    pub fn from_vk(constraint_vk: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_CL_CONSTRAINT_COMMIT");
        hasher.update(constraint_vk);
        let constraint_cm: [u8; 32] = hasher.finalize().into();

        Self(constraint_cm)
    }
}

pub fn derive_unit(unit: &str) -> Unit {
    let mut hasher = Sha256::new();
    hasher.update(b"NOMOS_CL_UNIT");
    hasher.update(unit.as_bytes());
    let unit: Unit = hasher.finalize().into();
    unit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NoteCommitment(pub [u8; 32]);

impl NoteCommitment {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct NoteWitness {
    pub value: u64,
    pub unit: Unit,
    pub constraint: Constraint,
    pub state: [u8; 32],
    pub nonce: Nonce,
}

impl NoteWitness {
    pub fn new(
        value: u64,
        unit: Unit,
        constraint: Constraint,
        state: [u8; 32],
        nonce: Nonce,
    ) -> Self {
        Self {
            value,
            unit,
            constraint,
            state,
            nonce,
        }
    }

    pub fn basic(value: u64, unit: Unit, rng: impl RngCore) -> Self {
        let constraint = Constraint([0u8; 32]);
        let nonce = Nonce::random(rng);
        Self::new(value, unit, constraint, [0u8; 32], nonce)
    }

    pub fn stateless(value: u64, unit: Unit, constraint: Constraint, rng: impl RngCore) -> Self {
        Self::new(value, unit, constraint, [0u8; 32], Nonce::random(rng))
    }

    pub fn commit(&self, tag: &dyn AsRef<[u8]>, nf_pk: NullifierCommitment) -> NoteCommitment {
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_CL_NOTE_CM");
        hasher.update(tag.as_ref());

        // COMMIT TO BALANCE
        hasher.update(self.value.to_le_bytes());
        hasher.update(self.unit);
        // Important! we don't commit to the balance blinding factor as that may make the notes linkable.

        // COMMIT TO STATE
        hasher.update(self.state);

        // COMMIT TO CONSTRAINT
        hasher.update(self.constraint.0);

        // COMMIT TO NONCE
        hasher.update(self.nonce.as_bytes());

        // COMMIT TO NULLIFIER
        hasher.update(nf_pk.as_bytes());

        let commit_bytes: [u8; 32] = hasher.finalize().into();
        NoteCommitment(commit_bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nonce([u8; 32]);

impl Nonce {
    pub fn random(mut rng: impl RngCore) -> Self {
        let mut nonce = [0u8; 32];
        rng.fill_bytes(&mut nonce);
        Self(nonce)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::cl::nullifier::NullifierSecret;

    // #[test]
    // fn test_note_commit_permutations() {
    //     let (nmo, eth) = (derive_unit("NMO"), derive_unit("ETH"));

    //     let mut rng = rand::thread_rng();

    //     let nf_pk = NullifierSecret::random(&mut rng).commit();

    //     let reference_note = NoteWitness::basic(32, nmo, &mut rng);

    //     // different notes under same nullifier produce different commitments
    //     let mutation_tests = [
    //         NoteWitness {
    //             value: 12,
    //             ..reference_note
    //         },
    //         NoteWitness {
    //             unit: eth,
    //             ..reference_note
    //         },
    //         NoteWitness {
    //             constraint: Constraint::from_vk(&[1u8; 32]),
    //             ..reference_note
    //         },
    //         NoteWitness {
    //             state: [1u8; 32],
    //             ..reference_note
    //         },
    //         NoteWitness {
    //             nonce: Nonce::random(&mut rng),
    //             ..reference_note
    //         },
    //     ];

    //     for n in mutation_tests {
    //         assert_ne!(n.commit(nf_pk), reference_note.commit(nf_pk));
    //     }

    //     // commitment to same note with different nullifiers produce different commitments

    //     let other_nf_pk = NullifierSecret::random(&mut rng).commit();

    //     assert_ne!(
    //         reference_note.commit(nf_pk),
    //         reference_note.commit(other_nf_pk)
    //     );
    // }
}
