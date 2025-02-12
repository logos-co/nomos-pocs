use crate::{
    crust::{
        balance::Unit,
        note::NoteWitness,
        nullifier::{Nullifier, NullifierCommitment, NullifierSecret},
        Nonce, NoteCommitment,
    },
    mantle::ZoneId,
    Digest, Hash,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputWitness {
    pub note: NoteWitness,
    pub nf_sk: NullifierSecret,
    pub zone_id: ZoneId,
}

impl InputWitness {
    pub fn new(note: NoteWitness, nf_sk: NullifierSecret, zone_id: ZoneId) -> Self {
        Self {
            note,
            nf_sk,
            zone_id,
        }
    }

    pub fn from_output(output: OutputWitness, nf_sk: NullifierSecret) -> Self {
        assert_eq!(nf_sk.commit(), output.nf_pk);
        Self::new(output.note, nf_sk, output.zone_id)
    }

    pub fn evolved_nonce(&self, domain: &[u8]) -> Nonce {
        let mut hasher = Hash::new();
        hasher.update(b"NOMOS_COIN_EVOLVE");
        hasher.update(domain);
        hasher.update(self.nf_sk.0);
        hasher.update(self.note.commit(&self.zone_id, self.nf_sk.commit()).0);

        let nonce_bytes: [u8; 32] = hasher.finalize().into();
        Nonce::from_bytes(nonce_bytes)
    }

    pub fn evolve_output(&self, domain: &[u8]) -> OutputWitness {
        OutputWitness {
            note: NoteWitness {
                nonce: self.evolved_nonce(domain),
                ..self.note
            },
            nf_pk: self.nf_sk.commit(),
            zone_id: self.zone_id,
        }
    }

    pub fn nullifier(&self) -> Nullifier {
        Nullifier::new(&self.zone_id, self.nf_sk, self.note_commitment())
    }

    pub fn note_commitment(&self) -> NoteCommitment {
        self.note.commit(&self.zone_id, self.nf_sk.commit())
    }
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

    pub fn note_commitment(&self) -> NoteCommitment {
        self.note.commit(&self.zone_id, self.nf_pk)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintWitness {
    pub amount: u64,
    pub unit: Unit,
}

impl MintWitness {
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        bytes[0..32].copy_from_slice(&self.unit);
        bytes[32..].copy_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurnWitness {
    pub amount: u64,
    pub unit: Unit,
}

impl BurnWitness {
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        bytes[0..32].copy_from_slice(&self.unit);
        bytes[32..].copy_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}
