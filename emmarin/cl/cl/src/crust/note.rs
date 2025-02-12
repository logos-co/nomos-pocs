use crate::crust::{balance::Unit, nullifier::NullifierCommitment};
use crate::mantle::ZoneId;
use crate::{Digest, Hash};
use rand::RngCore;
use serde::{Deserialize, Serialize};

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
