pub mod mmr;

use cl::{balance::Unit, Constraint, NoteCommitment};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneMetadata {
    pub zone_constraint: Constraint,
    pub funds_constraint: Constraint,
    pub unit: Unit,
}

impl ZoneMetadata {
    pub fn id(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.zone_constraint.0);
        hasher.update(self.funds_constraint.0);
        hasher.update(self.unit);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateWitness {
    pub allowed_pks: Vec<NullifierCommitment>,
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
}

impl From<StateCommitment> for [u8; 32] {
    fn from(commitment: StateCommitment) -> [u8; 32] {
        commitment.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tx {
    Transfer(TransferPublic),
    Withdraw(WithdrawPublic),
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
