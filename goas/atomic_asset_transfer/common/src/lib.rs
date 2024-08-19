use cl::{balance::Unit, merkle, NoteCommitment};
use ed25519_dalek::{
    ed25519::{signature::SignerMut, SignatureBytes},
    Signature, SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH,
};
use once_cell::sync::Lazy;
use rand_core::CryptoRngCore;
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

pub type AccountId = [u8; PUBLIC_KEY_LENGTH];

// PLACEHOLDER: this is probably going to be NMO?
pub static ZONE_CL_FUNDS_UNIT: Lazy<Unit> = Lazy::new(|| cl::note::unit_point("NMO"));

pub fn new_account(mut rng: impl CryptoRngCore) -> SigningKey {
    SigningKey::generate(&mut rng)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneMetadata {
    pub zone_vk: [u8; 32],
    pub funds_vk: [u8; 32],
    pub unit: Unit,
}

impl ZoneMetadata {
    pub fn id(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.zone_vk);
        hasher.update(self.funds_vk);
        hasher.update(self.unit.compress().as_bytes());
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateWitness {
    pub balances: BTreeMap<AccountId, u64>,
    pub included_txs: Vec<Tx>,
    pub zone_metadata: ZoneMetadata,
}

impl StateWitness {
    pub fn commit(&self) -> StateCommitment {
        self.state_roots().commit()
    }

    pub fn state_roots(&self) -> StateRoots {
        StateRoots {
            tx_root: self.included_txs_root(),
            zone_id: self.zone_metadata.id(),
            balance_root: self.balances_root(),
        }
    }

    pub fn apply(self, tx: Tx) -> Self {
        let mut state = match tx {
            Tx::Withdraw(w) => self.withdraw(w),
            Tx::Deposit(d) => self.deposit(d),
        };

        state.included_txs.push(tx);

        state
    }

    fn withdraw(mut self, w: Withdraw) -> Self {
        let Withdraw { from, amount } = w;

        let from_balance = self.balances.entry(from).or_insert(0);
        *from_balance = from_balance
            .checked_sub(amount)
            .expect("insufficient funds in account");

        self
    }

    fn deposit(mut self, d: Deposit) -> Self {
        let Deposit { to, amount } = d;

        let to_balance = self.balances.entry(to).or_insert(0);
        *to_balance += to_balance
            .checked_add(amount)
            .expect("overflow in account balance");

        self
    }

    pub fn included_txs_root(&self) -> [u8; 32] {
        merkle::root::<MAX_TXS>(self.included_tx_merkle_leaves())
    }

    pub fn included_tx_witness(&self, idx: usize) -> IncludedTxWitness {
        let tx = *self.included_txs.get(idx).unwrap();
        let path = merkle::path(self.included_tx_merkle_leaves(), idx);
        IncludedTxWitness { tx, path }
    }

    pub fn balances_root(&self) -> [u8; 32] {
        let balance_bytes = Vec::from_iter(self.balances.iter().map(|(owner, balance)| {
            let mut bytes: Vec<u8> = vec![];
            bytes.extend(owner);
            bytes.extend(balance.to_le_bytes());
            bytes
        }));
        let balance_merkle_leaves = cl::merkle::padded_leaves(&balance_bytes);
        merkle::root::<MAX_BALANCES>(balance_merkle_leaves)
    }

    pub fn total_balance(&self) -> u64 {
        self.balances.values().sum()
    }

    fn included_tx_merkle_leaves(&self) -> [[u8; 32]; MAX_TXS] {
        let tx_bytes = self
            .included_txs
            .iter()
            .map(|t| t.to_bytes())
            .collect::<Vec<_>>();
        merkle::padded_leaves(&tx_bytes)
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
}

impl Withdraw {
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0; 40];
        bytes[0..PUBLIC_KEY_LENGTH].copy_from_slice(&self.from);
        bytes[PUBLIC_KEY_LENGTH..PUBLIC_KEY_LENGTH + 8].copy_from_slice(&self.amount.to_le_bytes());
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
    pub fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0; 40];
        bytes[0..PUBLIC_KEY_LENGTH].copy_from_slice(&self.to);
        bytes[PUBLIC_KEY_LENGTH..PUBLIC_KEY_LENGTH + 8].copy_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBoundTx {
    pub bound_tx: BoundTx,
    #[serde(with = "serde_arrays")]
    pub sig: SignatureBytes,
}

impl SignedBoundTx {
    pub fn sign(bound_tx: BoundTx, signing_key: &mut SigningKey) -> Self {
        let msg = bound_tx.to_bytes();
        let sig = signing_key.sign(&msg).to_bytes();

        Self { bound_tx, sig }
    }

    pub fn verify_and_unwrap(&self) -> BoundTx {
        let msg = self.bound_tx.to_bytes();

        let sig = Signature::from_bytes(&self.sig);
        let vk = self.bound_tx.tx.verifying_key();
        vk.verify_strict(&msg, &sig).expect("Invalid tx signature");

        self.bound_tx
    }
}

/// A Tx that is executed in the zone if and only if the bind is
/// present is the same partial transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundTx {
    pub tx: Tx,
    pub bind: NoteCommitment,
}

impl BoundTx {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.tx.to_bytes());
        bytes.extend(self.bind.as_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tx {
    Withdraw(Withdraw),
    Deposit(Deposit),
}

impl Tx {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Tx::Withdraw(withdraw) => withdraw.to_bytes().to_vec(),
            Tx::Deposit(deposit) => deposit.to_bytes().to_vec(),
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        match self {
            Tx::Withdraw(w) => VerifyingKey::from_bytes(&w.from).unwrap(),
            Tx::Deposit(d) => VerifyingKey::from_bytes(&d.to).unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludedTxWitness {
    pub tx: Tx,
    pub path: Vec<merkle::PathNode>,
}

impl IncludedTxWitness {
    pub fn tx_root(&self) -> [u8; 32] {
        let leaf = merkle::leaf(&self.tx.to_bytes());
        merkle::path_root(leaf, &self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRoots {
    pub tx_root: [u8; 32],
    pub zone_id: [u8; 32],
    pub balance_root: [u8; 32],
}

impl StateRoots {
    /// Merkle tree over: [txs, zoneid, balances]
    pub fn commit(&self) -> StateCommitment {
        let leaves = cl::merkle::padded_leaves::<4>(&[
            self.tx_root.to_vec(),
            self.zone_id.to_vec(),
            self.balance_root.to_vec(),
        ]);
        StateCommitment(cl::merkle::root(leaves))
    }
}
