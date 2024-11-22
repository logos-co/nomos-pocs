/// This module defines the partial transaction structure.
///
/// Partial transactions, as the name suggests, are transactions
/// which on their own may not balance (i.e. \sum inputs != \sum outputs)
use crate::{
    cl::{
        note::{Constraint, NoteWitness},
        nullifier::{Nullifier, NullifierSecret},
        Nonce, NoteCommitment, OutputWitness,
    },
    zone_layer::notes::ZoneId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    pub nullifier: Nullifier,
    pub constraint: Constraint,
    pub zone_id: ZoneId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputWitness {
    pub note: NoteWitness,
    pub nf_sk: NullifierSecret,
}

impl InputWitness {
    pub fn new(note: NoteWitness, nf_sk: NullifierSecret) -> Self {
        Self { note, nf_sk }
    }

    pub fn from_output(output: OutputWitness, nf_sk: NullifierSecret) -> Self {
        assert_eq!(nf_sk.commit(), output.nf_pk);
        Self::new(output.note, nf_sk)
    }

    pub fn public(output: OutputWitness) -> Self {
        let nf_sk = NullifierSecret::zero();
        assert_eq!(nf_sk.commit(), output.nf_pk); // ensure the output was a public UTXO
        Self::new(output.note, nf_sk)
    }

    pub fn evolved_nonce(&self, tag: &dyn AsRef<[u8]>, domain: &[u8]) -> Nonce {
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_COIN_EVOLVE");
        hasher.update(domain);
        hasher.update(self.nf_sk.0);
        hasher.update(self.note.commit(tag, self.nf_sk.commit()).0);

        let nonce_bytes: [u8; 32] = hasher.finalize().into();
        Nonce::from_bytes(nonce_bytes)
    }

    pub fn evolve_output(&self, tag: &dyn AsRef<[u8]>, domain: &[u8]) -> OutputWitness {
        OutputWitness {
            note: NoteWitness {
                nonce: self.evolved_nonce(tag, domain),
                ..self.note
            },
            nf_pk: self.nf_sk.commit(),
        }
    }

    pub fn nullifier(&self, tag: &dyn AsRef<[u8]>) -> Nullifier {
        Nullifier::new(tag, self.nf_sk, self.note_commitment(tag))
    }

    pub fn commit(&self, zone_id: ZoneId) -> Input {
        Input {
            nullifier: self.nullifier(&zone_id),
            constraint: self.note.constraint,
            zone_id,
        }
    }

    pub fn note_commitment(&self, tag: &dyn AsRef<[u8]>) -> NoteCommitment {
        self.note.commit(tag, self.nf_sk.commit())
    }
}

impl Input {
    pub fn to_bytes(&self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[..32].copy_from_slice(self.nullifier.as_bytes());
        bytes[32..64].copy_from_slice(&self.constraint.0);
        bytes[64..96].copy_from_slice(&self.zone_id);
        bytes
    }
}
