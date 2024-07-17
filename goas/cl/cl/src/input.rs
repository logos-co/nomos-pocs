/// This module defines the partial transaction structure.
///
/// Partial transactions, as the name suggests, are transactions
/// which on their own may not balance (i.e. \sum inputs != \sum outputs)
use crate::{
    balance::Balance,
    note::{DeathCommitment, NoteWitness},
    nullifier::{Nullifier, NullifierNonce, NullifierSecret},
    BalanceWitness,
};
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Input {
    pub nullifier: Nullifier,
    pub balance: Balance,
    pub death_cm: DeathCommitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputWitness {
    pub note: NoteWitness,
    pub utxo_balance_blinding: BalanceWitness,
    pub balance_blinding: BalanceWitness,
    pub nf_sk: NullifierSecret,
    pub nonce: NullifierNonce,
}

impl InputWitness {
    pub fn random(
        output: crate::OutputWitness,
        nf_sk: NullifierSecret,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        assert_eq!(nf_sk.commit(), output.nf_pk);
        Self {
            note: output.note,
            utxo_balance_blinding: output.balance_blinding,
            balance_blinding: BalanceWitness::random(&mut rng),
            nf_sk,
            nonce: output.nonce,
        }
    }

    pub fn nullifier(&self) -> Nullifier {
        Nullifier::new(self.nf_sk, self.nonce)
    }

    pub fn commit(&self) -> Input {
        Input {
            nullifier: self.nullifier(),
            balance: self.balance_blinding.commit(&self.note),
            death_cm: self.note.death_commitment(),
        }
    }

    pub fn to_output(&self) -> crate::OutputWitness {
        crate::OutputWitness {
            note: self.note,
            balance_blinding: self.utxo_balance_blinding,
            nf_pk: self.nf_sk.commit(),
            nonce: self.nonce,
        }
    }
}

impl Input {
    pub fn to_bytes(&self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[..32].copy_from_slice(self.nullifier.as_bytes());
        bytes[32..64].copy_from_slice(&self.balance.to_bytes());
        bytes[64..96].copy_from_slice(&self.death_cm.0);
        bytes
    }
}
