/// This module defines the partial transaction structure.
///
/// Partial transactions, as the name suggests, are transactions
/// which on their own may not balance (i.e. \sum inputs != \sum outputs)
use crate::{
    note::{DeathCommitment, NoteWitness},
    nullifier::{Nullifier, NullifierNonce, NullifierSecret},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    pub nullifier: Nullifier,
    pub death_cm: DeathCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputWitness {
    pub note: NoteWitness,
    pub nf_sk: NullifierSecret,
    pub nonce: NullifierNonce,
}

impl InputWitness {
    pub fn from_output(output: crate::OutputWitness, nf_sk: NullifierSecret) -> Self {
        assert_eq!(nf_sk.commit(), output.nf_pk);
        Self {
            note: output.note,
            nf_sk,
            nonce: output.nonce,
        }
    }

    pub fn public(output: crate::OutputWitness) -> Self {
        let nf_sk = NullifierSecret::zero();
        assert_eq!(nf_sk.commit(), output.nf_pk); // ensure the output was a public UTXO
        Self {
            note: output.note,
            nf_sk,
            nonce: output.nonce,
        }
    }

    pub fn evolved_nonce(&self, domain: &[u8]) -> NullifierNonce {
        self.nonce.evolve(domain, &self.nf_sk, &self.note)
    }

    pub fn evolve_output(&self, domain: &[u8]) -> crate::OutputWitness {
        crate::OutputWitness {
            note: self.note,
            nf_pk: self.nf_sk.commit(),
            nonce: self.evolved_nonce(domain),
        }
    }

    pub fn nullifier(&self) -> Nullifier {
        Nullifier::new(self.nf_sk, self.note_commitment())
    }

    pub fn commit(&self) -> Input {
        Input {
            nullifier: self.nullifier(),
            death_cm: self.note.death_commitment(),
        }
    }

    pub fn note_commitment(&self) -> crate::NoteCommitment {
        self.note.commit(self.nf_sk.commit(), self.nonce)
    }
}

impl Input {
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(self.nullifier.as_bytes());
        bytes[32..64].copy_from_slice(&self.death_cm.0);
        bytes
    }
}
