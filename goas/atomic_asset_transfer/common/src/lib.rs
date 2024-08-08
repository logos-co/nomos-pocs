use cl::{balance::Unit, nullifier::NullifierCommitment};
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
pub struct StateCommitment(pub [u8; 32]);

pub type AccountId = u32;

// PLACEHOLDER: this is probably going to be NMO?
pub static ZONE_CL_FUNDS_UNIT: Lazy<Unit> = Lazy::new(|| cl::note::unit_point("NMO"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateWitness {
    pub balances: BTreeMap<AccountId, u64>,
    pub included_txs: Vec<Input>,
    pub zone_metadata: ZoneMetadata,
    pub nonce: [u8; 32],
}

impl StateWitness {
    /// Merkle tree over:
    ///                  root
    ///              /        \
    ///            io          state
    ///          /   \        /     \
    ///      nonce   txs   zoneid  balances
    pub fn commit(&self) -> StateCommitment {
        let root = cl::merkle::root([
            self.nonce,
            self.included_txs_root(),
            self.zone_metadata.id(),
            self.balances_root(),
        ]);

        StateCommitment(root)
    }

    pub fn withdraw(mut self, w: Withdraw) -> Self {
        self.included_txs.push(Input::Withdraw(w));

        let Withdraw {
            from,
            amount,
            to: _,
        } = w;

        let from_balance = self.balances.entry(from).or_insert(0);
        *from_balance = from_balance
            .checked_sub(amount)
            .expect("insufficient funds in account");

        self
    }

    pub fn deposit(mut self, d: Deposit) -> Self {
        self.included_txs.push(Input::Deposit(d));

        let Deposit { to, amount } = d;

        let to_balance = self.balances.entry(to).or_insert(0);
        *to_balance += to_balance
            .checked_add(amount)
            .expect("overflow in account balance");
        self
    }

    pub fn included_txs_root(&self) -> [u8; 32] {
        // this is a placeholder
        let tx_bytes = [vec![0u8; 32]];
        let tx_merkle_leaves = cl::merkle::padded_leaves(&tx_bytes);
        cl::merkle::root::<MAX_TXS>(tx_merkle_leaves)
    }

    pub fn balances_root(&self) -> [u8; 32] {
        let balance_bytes = Vec::from_iter(self.balances.iter().map(|(owner, balance)| {
            let mut bytes: Vec<u8> = vec![];
            bytes.extend(owner.to_le_bytes());
            bytes.extend(balance.to_le_bytes());
            bytes
        }));
        let balance_merkle_leaves = cl::merkle::padded_leaves(&balance_bytes);
        cl::merkle::root::<MAX_BALANCES>(balance_merkle_leaves)
    }

    pub fn total_balance(&self) -> u64 {
        self.balances.values().sum()
    }

    pub fn evolve_nonce(self) -> Self {
        let updated_nonce = {
            let mut hasher = Sha256::new();
            hasher.update(&self.nonce);
            hasher.update(b"NOMOS_ZONE_NONCE_EVOLVE");
            hasher.finalize().into()
        };
        Self {
            nonce: updated_nonce,
            ..self
        }
    }
}

impl From<StateCommitment> for [u8; 32] {
    fn from(commitment: StateCommitment) -> [u8; 32] {
        commitment.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Withdraw {
    pub from: AccountId,
    pub amount: u64,
    pub to: NullifierCommitment,
}

impl Withdraw {
    pub fn to_bytes(&self) -> [u8; 44] {
        let mut bytes = [0; 44];
        bytes[0..4].copy_from_slice(&self.from.to_le_bytes());
        bytes[4..12].copy_from_slice(&self.amount.to_le_bytes());
        bytes[12..44].copy_from_slice(self.to.as_bytes());
        bytes
    }
}

/// A deposit of funds into the zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deposit {
    pub to: AccountId,
    pub amount: u64,
}

impl Deposit {
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0; 32];
        bytes[0..4].copy_from_slice(&self.to.to_le_bytes());
        bytes[4..12].copy_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Input {
    Withdraw(Withdraw),
    Deposit(Deposit),
}
