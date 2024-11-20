use serde::{Deserialize, Serialize};

use crate::cl::{
    note::{NoteCommitment, NoteWitness},
    nullifier::NullifierCommitment,
    NullifierSecret,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub note_comm: NoteCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputWitness {
    pub note: NoteWitness,
    pub nf_pk: NullifierCommitment,
}

impl OutputWitness {
    pub fn new(note: NoteWitness, nf_pk: NullifierCommitment) -> Self {
        Self { note, nf_pk }
    }

    pub fn public(note: NoteWitness) -> Self {
        let nf_pk = NullifierSecret::zero().commit();
        Self { note, nf_pk }
    }

    pub fn commit_note(&self, tag: &dyn AsRef<[u8]>) -> NoteCommitment {
        self.note.commit(tag, self.nf_pk)
    }

    pub fn commit(&self, tag: &dyn AsRef<[u8]>) -> Output {
        Output {
            note_comm: self.commit_note(tag),
        }
    }
}

impl Output {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.note_comm.0
    }
}
