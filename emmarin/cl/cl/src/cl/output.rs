use serde::{Deserialize, Serialize};

use crate::{
    cl::{
        note::{NoteCommitment, NoteWitness},
        nullifier::NullifierCommitment,
        NullifierSecret,
    },
    zone_layer::notes::ZoneId,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub zone_id: ZoneId,
    pub note_comm: NoteCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputWitness {
    pub note: NoteWitness,
    pub nf_pk: NullifierCommitment,
    pub zone_id: ZoneId,
}

impl OutputWitness {
    pub fn new(note: NoteWitness, nf_pk: NullifierCommitment, zone_id: ZoneId) -> Self {
        Self {
            note,
            nf_pk,
            zone_id,
        }
    }

    pub fn public(note: NoteWitness, zone_id: ZoneId) -> Self {
        let nf_pk = NullifierSecret::zero().commit();
        Self {
            note,
            nf_pk,
            zone_id,
        }
    }

    pub fn commit_note(&self) -> NoteCommitment {
        self.note.commit(&self.zone_id, self.nf_pk)
    }

    pub fn commit(&self) -> Output {
        Output {
            zone_id: self.zone_id,
            note_comm: self.commit_note(),
        }
    }
}

impl Output {
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&self.zone_id);
        bytes[32..].copy_from_slice(&self.note_comm.0);
        bytes
    }
}
