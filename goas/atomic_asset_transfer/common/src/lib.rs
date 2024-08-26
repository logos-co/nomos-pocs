pub mod mmr;

use cl::{balance::Unit, NoteCommitment};
use ed25519_dalek::{
    ed25519::{signature::SignerMut, SignatureBytes},
    Signature, SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH,
};
use mmr::{MMRProof, MMR};
use once_cell::sync::Lazy;
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

// state of the zone
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct StateCommitment(pub [u8; 32]);

pub type AccountId = [u8; PUBLIC_KEY_LENGTH];

// PLACEHOLDER: this is probably going to be NMO?
pub static ZONE_CL_FUNDS_UNIT: Lazy<Unit> = Lazy::new(|| cl::note::derive_unit("NMO"));

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
        hasher.update(self.unit);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateWitness {
    pub balances: BTreeMap<AccountId, u64>,
    pub included_txs: MMR,
    pub zone_metadata: ZoneMetadata,
}

impl StateWitness {
    pub fn commit(&self) -> StateCommitment {
        self.state_roots().commit()
    }

    pub fn state_roots(&self) -> StateRoots {
        StateRoots {
            included_txs: self.included_txs.clone(),
            zone_id: self.zone_metadata.id(),
            balance_root: self.balances_root(),
        }
    }

    pub fn apply(self, tx: Tx) -> (Self, IncludedTxWitness) {
        let mut state = match tx {
            Tx::Withdraw(w) => self.withdraw(w),
            Tx::Deposit(d) => self.deposit(d),
        };

        let inclusion_proof = state.included_txs.push(&tx.to_bytes());
        let tx_inclusion_proof = IncludedTxWitness {
            tx,
            proof: inclusion_proof,
        };

        (state, tx_inclusion_proof)
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

    pub fn balances_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_BALANCES_ROOT");

        for (k, v) in self.balances.iter() {
            hasher.update(k);
            hasher.update(&v.to_le_bytes());
        }

        hasher.finalize().into()
    }

    pub fn total_balance(&self) -> u64 {
        self.balances.values().sum()
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
    pub proof: MMRProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRoots {
    pub included_txs: MMR,
    pub zone_id: [u8; 32],
    pub balance_root: [u8; 32],
}

impl StateRoots {
    pub fn verify_tx_inclusion(&self, tx_inclusion: &IncludedTxWitness) -> bool {
        self.included_txs
            .verify_proof(&tx_inclusion.tx.to_bytes(), &tx_inclusion.proof)
    }

    /// Commitment to the state roots
    pub fn commit(&self) -> StateCommitment {
        let mut hasher = Sha256::new();
        hasher.update(self.included_txs.commit());
        hasher.update(self.zone_id);
        hasher.update(self.balance_root);
        StateCommitment(hasher.finalize().into())
    }
}
