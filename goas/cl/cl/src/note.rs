use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    balance::Unit,
    nullifier::{NullifierCommitment, NullifierNonce},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeathCommitment(pub [u8; 32]);

pub fn death_commitment(death_constraint: &[u8]) -> DeathCommitment {
    let mut hasher = Sha256::new();
    hasher.update(b"NOMOS_CL_DEATH_COMMIT");
    hasher.update(death_constraint);
    let death_cm: [u8; 32] = hasher.finalize().into();

    DeathCommitment(death_cm)
}

pub fn unit_point(unit: &str) -> Unit {
    crate::crypto::hash_to_curve(unit.as_bytes())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NoteCommitment([u8; 32]);

impl NoteCommitment {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

// TODO: Rename Note to NoteWitness and NoteCommitment to Note

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct NoteWitness {
    pub value: u64,
    pub unit: Unit,
    pub death_constraint: [u8; 32], // death constraint verification key
    pub state: [u8; 32],
}

impl NoteWitness {
    pub fn new(
        value: u64,
        unit: impl Into<String>,
        death_constraint: [u8; 32],
        state: [u8; 32],
    ) -> Self {
        Self {
            value,
            unit: unit_point(&unit.into()),
            death_constraint,
            state,
        }
    }

    pub fn basic(value: u64, unit: impl Into<String>) -> Self {
        Self::new(value, unit, [0u8; 32], [0u8; 32])
    }

    pub fn stateless(value: u64, unit: impl Into<String>, death_constraint: [u8; 32]) -> Self {
        Self::new(value, unit, death_constraint, [0u8; 32])
    }

    pub fn commit(&self, nf_pk: NullifierCommitment, nonce: NullifierNonce) -> NoteCommitment {
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_CL_NOTE_COMMIT");

        // COMMIT TO BALANCE
        hasher.update(self.value.to_le_bytes());
        hasher.update(self.unit.compress().to_bytes());
        // Important! we don't commit to the balance blinding factor as that may make the notes linkable.

        // COMMIT TO STATE
        hasher.update(self.state);

        // COMMIT TO DEATH CONSTRAINT
        hasher.update(self.death_constraint);

        // COMMIT TO NULLIFIER
        hasher.update(nf_pk.as_bytes());
        hasher.update(nonce.as_bytes());

        let commit_bytes: [u8; 32] = hasher.finalize().into();
        NoteCommitment(commit_bytes)
    }

    pub fn death_commitment(&self) -> DeathCommitment {
        death_commitment(&self.death_constraint)
    }
}

#[cfg(test)]
mod test {
    use crate::nullifier::NullifierSecret;

    use super::*;

    #[test]
    fn test_note_commit_permutations() {
        let mut rng = rand::thread_rng();

        let nf_pk = NullifierSecret::random(&mut rng).commit();
        let nf_nonce = NullifierNonce::random(&mut rng);

        let reference_note = NoteWitness::basic(32, "NMO");

        // different notes under same nullifier produce different commitments
        let mutation_tests = [
            NoteWitness {
                value: 12,
                ..reference_note
            },
            NoteWitness {
                unit: unit_point("ETH"),
                ..reference_note
            },
            NoteWitness {
                death_constraint: [1u8; 32],
                ..reference_note
            },
            NoteWitness {
                state: [1u8; 32],
                ..reference_note
            },
        ];

        for n in mutation_tests {
            assert_ne!(
                n.commit(nf_pk, nf_nonce),
                reference_note.commit(nf_pk, nf_nonce)
            );
        }

        // commitment to same note with different nullifiers produce different commitments

        let other_nf_pk = NullifierSecret::random(&mut rng).commit();
        let other_nf_nonce = NullifierNonce::random(&mut rng);

        assert_ne!(
            reference_note.commit(nf_pk, nf_nonce),
            reference_note.commit(other_nf_pk, nf_nonce)
        );
        assert_ne!(
            reference_note.commit(nf_pk, nf_nonce),
            reference_note.commit(nf_pk, other_nf_nonce)
        );
        assert_ne!(
            reference_note.commit(nf_pk, nf_nonce),
            reference_note.commit(other_nf_pk, other_nf_nonce)
        );
    }
}
