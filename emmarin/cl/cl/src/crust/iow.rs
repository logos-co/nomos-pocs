use crate::{
    crust::{
        balance::{Unit, UnitWitness},
        nullifier::{Nullifier, NullifierCommitment, NullifierSecret},
    },
    mantle::ZoneId,
    Digest, Hash,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputWitness {
    pub state: [u8; 32],
    pub value: u64,
    pub unit_witness: UnitWitness,
    pub nonce: Nonce,
    pub zone_id: ZoneId,
    pub nf_sk: NullifierSecret,
}

impl InputWitness {
    pub fn from_output(
        output: OutputWitness,
        nf_sk: NullifierSecret,
        unit_witness: UnitWitness,
    ) -> Self {
        assert_eq!(nf_sk.commit(), output.nf_pk);
        assert_eq!(unit_witness.unit(), output.unit);
        Self {
            state: output.state,
            value: output.value,
            unit_witness,
            nonce: output.nonce,
            zone_id: output.zone_id,
            nf_sk,
        }
    }

    pub fn evolved_nonce(&self, domain: &[u8]) -> Nonce {
        let mut hasher = Hash::new();
        hasher.update(b"NOMOS_COIN_EVOLVE");
        hasher.update(domain);
        hasher.update(self.nf_sk.0);
        hasher.update(self.note_commitment().0);

        let nonce_bytes: [u8; 32] = hasher.finalize().into();
        Nonce::from_bytes(nonce_bytes)
    }

    pub fn evolve_output(&self, domain: &[u8]) -> OutputWitness {
        OutputWitness {
            state: self.state,
            value: self.value,
            unit: self.unit_witness.unit(),
            nonce: self.evolved_nonce(domain),
            zone_id: self.zone_id,
            nf_pk: self.nf_sk.commit(),
        }
    }

    pub fn nullifier(&self) -> Nullifier {
        Nullifier::new(&self.zone_id, self.nf_sk, self.note_commitment())
    }

    pub fn note_commitment(&self) -> NoteCommitment {
        NoteCommitment::commit(
            self.state,
            self.value,
            self.unit_witness.unit(),
            self.nonce,
            self.zone_id,
            self.nf_sk.commit(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputWitness {
    pub state: [u8; 32],
    pub value: u64,
    pub unit: Unit,
    pub nonce: Nonce,
    pub zone_id: ZoneId,
    pub nf_pk: NullifierCommitment,
}

impl OutputWitness {
    pub fn note_commitment(&self) -> NoteCommitment {
        NoteCommitment::commit(
            self.state,
            self.value,
            self.unit,
            self.nonce,
            self.zone_id,
            self.nf_pk,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NoteCommitment(pub [u8; 32]);

impl NoteCommitment {
    fn commit(
        state: [u8; 32],
        value: u64,
        unit: Unit,
        nonce: Nonce,
        zone_id: ZoneId,
        nf_pk: NullifierCommitment,
    ) -> Self {
        let mut hasher = Hash::new();
        hasher.update(b"NOMOS_NOTE_CM");
        hasher.update(state);
        hasher.update(value.to_le_bytes());
        hasher.update(unit);
        hasher.update(nonce.as_bytes());
        hasher.update(nf_pk.as_bytes());
        hasher.update(zone_id);
        let commit_bytes: [u8; 32] = hasher.finalize().into();
        Self(commit_bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintWitness {
    pub amount: u64,
    pub unit: UnitWitness,
    pub salt: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurnWitness {
    pub amount: u64,
    pub unit: UnitWitness,
    pub salt: [u8; 16],
}
