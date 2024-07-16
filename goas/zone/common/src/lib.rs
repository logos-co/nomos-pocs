use cl::nullifier::{Nullifier, NullifierCommitment};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// TODO: sparse merkle tree
pub const MAX_BALANCES: usize = 1 << 8;
pub const MAX_TXS: usize = 1 << 8;
pub const MAX_EVENTS: usize = 1 << 8;

// state of the zone
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct StateCommitment([u8; 32]);

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
        let tx_bytes = Vec::from_iter(
            self.included_txs
                .iter()
                .map(Input::to_bytes)
                .map(Vec::from_iter),
        );
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
    pub from: u32,
    pub amount: u32,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Input {
    Withdraw(Withdraw),
}

impl Input {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Input::Withdraw(withdraw) => withdraw.to_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Event {
    Spend(proof_statements::zone_funds::Spend),
}

impl Event {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Event::Spend(spend) => spend.to_bytes().to_vec(),
        }
    }
}
