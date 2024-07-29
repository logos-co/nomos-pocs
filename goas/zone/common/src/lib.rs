use cl::{
    balance::Unit,
    crypto,
    nullifier::{Nullifier, NullifierCommitment},
    output::OutputWitness,
};
use once_cell::sync::Lazy;
use proof_statements::ptx::PartialTxInputPrivate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// TODO: sparse merkle tree
pub const MAX_BALANCES: usize = 1 << 8;
pub const MAX_TXS: usize = 1 << 8;
pub const MAX_EVENTS: usize = 1 << 8;

// state of the zone
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct StateCommitment([u8; 32]);

pub type AccountId = u32;

// PLACEHOLDER: replace with the death constraint vk of the zone funds
pub const ZONE_FUNDS_VK: [u8; 32] = [0; 32];
// PLACEHOLDER: this is probably going to be NMO?
pub static ZONE_CL_FUNDS_UNIT: Lazy<Unit> = Lazy::new(|| crypto::hash_to_curve(b"NMO"));
// PLACEHOLDER
pub static ZONE_UNIT: Lazy<Unit> = Lazy::new(|| crypto::hash_to_curve(b"ZONE_UNIT"));
// PLACEHOLDER
pub const ZONE_NF_PK: NullifierCommitment = NullifierCommitment::from_bytes([0; 32]);

#[derive(Clone, Serialize, Deserialize)]
pub struct StateWitness {
    pub balances: BTreeMap<u32, u32>,
    pub included_txs: Vec<Input>,
    pub output_events: Vec<Event>,
}

impl StateWitness {
    pub fn commit(&self) -> StateCommitment {
        let root = self.balances_root();
        let root = cl::merkle::node(self.events_root(), root);
        let root = cl::merkle::node(self.included_txs_root(), root);
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
    /// Zone funds are public so we don't need to keep this private
    /// The amount of funds being deposited and the account they are being deposited to
    /// is derived from the note itself
    pub deposit: PartialTxInputPrivate,
    /// Root of merkle tree over ptx inputs
    pub inputs_root: [u8; 32],
    pub zone_note: OutputWitness,
    pub zone_funds: OutputWitness,
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
