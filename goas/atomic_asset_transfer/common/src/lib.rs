use cl::{
    balance::Unit,
    crypto,
    input::InputWitness,
    nullifier::{Nullifier, NullifierCommitment},
    output::OutputWitness,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

// TODO: sparse merkle tree
pub const MAX_BALANCES: usize = 1 << 8;
pub const MAX_TXS: usize = 1 << 8;
pub const MAX_EVENTS: usize = 1 << 8;

// state of the zone
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct StateCommitment([u8; 32]);

pub type AccountId = u32;

// PLACEHOLDER: this is probably going to be NMO?
pub static ZONE_CL_FUNDS_UNIT: Lazy<Unit> = Lazy::new(|| crypto::hash_to_curve(b"NMO"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneMetadata {
    pub zone_vk: [u8; 32],
    pub funds_vk: [u8; 32],
    pub unit: Unit,
}

impl ZoneMetadata {
    pub fn id(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.zone_vk);
        hasher.update(&self.funds_vk);
        hasher.update(self.unit.compress().as_bytes());
        hasher.finalize().into()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StateWitness {
    pub balances: BTreeMap<u32, u32>,
    pub included_txs: Vec<Input>,
    pub output_events: Vec<Event>,
    pub zone_metadata: ZoneMetadata,
}

impl StateWitness {
    /// Merkle tree over:
    ///                  root
    ///              /        \
    ///            io          state
    ///          /   \        /     \
    ///      events   txs   zoneid  balances
    pub fn commit(&self) -> StateCommitment {
        let root = cl::merkle::root([
            self.events_root(),
            self.included_txs_root(),
            self.zone_metadata.id(),
            self.balances_root(),
        ]);

        StateCommitment(root)
    }

    fn events_root(&self) -> [u8; 32] {
        let event_bytes = Vec::from_iter(
            self.output_events
                .iter()
                .map(Event::to_bytes)
                .map(Vec::from_iter),
        );
        let event_merkle_leaves = cl::merkle::padded_leaves(&event_bytes);
        cl::merkle::root::<MAX_EVENTS>(event_merkle_leaves)
    }

    fn included_txs_root(&self) -> [u8; 32] {
        // this is a placeholder
        let tx_bytes = [vec![0u8; 32]];
        let tx_merkle_leaves = cl::merkle::padded_leaves(&tx_bytes);
        cl::merkle::root::<MAX_TXS>(tx_merkle_leaves)
    }

    fn balances_root(&self) -> [u8; 32] {
        let balance_bytes = Vec::from_iter(self.balances.iter().map(|(k, v)| {
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&k.to_le_bytes());
            bytes[8..].copy_from_slice(&v.to_le_bytes());
            bytes.to_vec()
        }));
        let balance_merkle_leaves = cl::merkle::padded_leaves(&balance_bytes);
        cl::merkle::root::<MAX_BALANCES>(balance_merkle_leaves)
    }
}

impl From<StateCommitment> for [u8; 32] {
    fn from(commitment: StateCommitment) -> [u8; 32] {
        commitment.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Withdraw {
    pub from: AccountId,
    pub amount: AccountId,
    pub to: NullifierCommitment,
    pub nf: Nullifier,
}

impl Withdraw {
    pub fn to_bytes(&self) -> [u8; 72] {
        let mut bytes = [0; 72];
        bytes[0..4].copy_from_slice(&self.from.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.amount.to_le_bytes());
        bytes[8..40].copy_from_slice(self.to.as_bytes());
        bytes[40..72].copy_from_slice(self.nf.as_bytes());
        bytes
    }
}

/// A deposit of funds into the zone
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    /// The note that is used to deposit funds into the zone
    pub deposit: InputWitness,

    // This zone state note
    pub zone_note_in: InputWitness,
    pub zone_note_out: OutputWitness,

    // The zone funds note
    pub zone_funds_in: InputWitness,
    pub zone_funds_out: OutputWitness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Withdraw(Withdraw),
    Deposit(Deposit),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Event {
    Spend(goas_proof_statements::zone_funds::Spend),
}

impl Event {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Event::Spend(spend) => spend.to_bytes().to_vec(),
        }
    }
}
