use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};

use crate::{
    note::{NoteCommitment, NoteWitness},
    nullifier::{NullifierCommitment, NullifierNonce},
    NullifierSecret,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub note_comm: NoteCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputWitness {
    pub note: NoteWitness,
    pub nf_pk: NullifierCommitment,
    pub nonce: NullifierNonce,
}

impl OutputWitness {
    pub fn random(
        note: NoteWitness,
        owner: NullifierCommitment,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        Self {
            note,
            nf_pk: owner,
            nonce: NullifierNonce::random(&mut rng),
        }
    }

    pub fn public(note: NoteWitness, nonce: NullifierNonce) -> Self {
        Self {
            note,
            nf_pk: NullifierSecret::zero().commit(),
            nonce,
        }
    }

    pub fn commit_note(&self) -> NoteCommitment {
        self.note.commit(self.nf_pk, self.nonce)
    }

    pub fn commit(&self) -> Output {
        Output {
            note_comm: self.commit_note(),
        }
    }
}

impl Output {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.note_comm.0
    }
}
