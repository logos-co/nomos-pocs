use crate::crust::{balance::Unit, nullifier::NullifierCommitment};
use crate::mantle::ZoneId;
use crate::{Digest, Hash};
use rand::RngCore;
use serde::{Deserialize, Serialize};

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
    pub state: [u8; 32],
    pub nonce: Nonce,
    pub zone_id: ZoneId,
}

impl NoteWitness {
    pub fn new(value: u64, unit: Unit, state: [u8; 32], nonce: Nonce, zone_id: ZoneId) -> Self {
        Self {
            value,
            unit,
            state,
            nonce,
            zone_id,
        }
    }

    pub fn stateless(value: u64, unit: Unit, zone_id: ZoneId, rng: impl RngCore) -> Self {
        Self::new(value, unit, [0u8; 32], Nonce::random(rng), zone_id)
    }

    pub fn commit(&self, nf_pk: NullifierCommitment) -> NoteCommitment {
        let mut hasher = Hash::new();
        hasher.update(b"NOMOS_CL_NOTE_CM");
        hasher.update(&self.zone_id);

        // COMMIT TO BALANCE
        hasher.update(self.value.to_le_bytes());
        hasher.update(self.unit);
        // Important! we don't commit to the balance blinding factor as that may make the notes linkable.

        // COMMIT TO STATE
        hasher.update(self.state);

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
    use super::*;
    use crate::cl::nullifier::NullifierSecret;

    #[test]
    fn test_note_commit_permutations() {
        let (nmo, eth) = ([0; 32], [1; 32]);

        let mut rng = rand::thread_rng();

        let nf_pk = NullifierSecret::random(&mut rng).commit();

        let reference_note = NoteWitness::basic(32, nmo, &mut rng);

        // different notes under same nullifier produce different commitments
        let mutation_tests = [
            NoteWitness {
                value: 12,
                ..reference_note
            },
            NoteWitness {
                unit: eth,
                ..reference_note
            },
            NoteWitness {
                covenant: Covenant::from_vk(&[1u8; 32]),
                ..reference_note
            },
            NoteWitness {
                state: [1u8; 32],
                ..reference_note
            },
            NoteWitness {
                nonce: Nonce::random(&mut rng),
                ..reference_note
            },
        ];

        for n in mutation_tests {
            assert_ne!(n.commit(nf_pk), reference_note.commit(nf_pk));
        }

        // commitment to same note with different nullifiers produce different commitments

        let other_nf_pk = NullifierSecret::random(&mut rng).commit();

        assert_ne!(
            reference_note.commit(nf_pk),
            reference_note.commit(other_nf_pk)
        );
    }
}
