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

    pub fn to_output(&self) -> OutputWitness {
        OutputWitness {
            state: self.state,
            value: self.value,
            unit: self.unit_witness.unit(),
            nonce: self.nonce,
            zone_id: self.zone_id,
            nf_pk: self.nf_sk.commit(),
        }
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
    pub fn new(
        value: u64,
        unit: Unit,
        nf_pk: NullifierCommitment,
        zone_id: ZoneId,
        rng: impl RngCore,
    ) -> Self {
        Self {
            state: [0; 32],
            value,
            unit,
            nonce: Nonce::random(rng),
            zone_id,
            nf_pk,
        }
    }

    pub fn reissue(input: InputWitness, rng: impl RngCore) -> Self {
        Self::new(
            input.value,
            input.unit_witness.unit(),
            input.nf_sk.commit(),
            input.zone_id,
            rng,
        )
    }

    pub fn spend_with_change(
        input: InputWitness,
        amount: u64,
        to_pk: NullifierCommitment,
        to_zone: ZoneId,
        mut rng: impl RngCore,
    ) -> (OutputWitness, OutputWitness) {
        assert!(input.value > amount);

        let transfer = OutputWitness::reissue(input, &mut rng)
            .set_value(amount)
            .set_nf_pk(to_pk)
            .set_zone(to_zone);

        let change = OutputWitness::reissue(input, &mut rng).set_value(input.value - amount);

        (transfer, change)
    }

    pub fn set_value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    pub fn set_nf_pk(mut self, nf_pk: NullifierCommitment) -> Self {
        self.nf_pk = nf_pk;
        self
    }

    pub fn set_zone(mut self, zone_id: ZoneId) -> Self {
        self.zone_id = zone_id;
        self
    }

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
