use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};

use crate::{
    balance::Balance,
    error::Error,
    note::{NoteCommitment, NoteWitness},
    nullifier::{NullifierCommitment, NullifierNonce},
    BalanceWitness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub note_comm: NoteCommitment,
    pub balance: Balance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputWitness {
    pub note: NoteWitness,
    pub balance_blinding: BalanceWitness,
    pub nf_pk: NullifierCommitment,
    pub nonce: NullifierNonce,
}

impl OutputWitness {
    pub fn random(
        note: NoteWitness,
        owner: NullifierCommitment,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        Self {
            note,
            balance_blinding: BalanceWitness::random(&mut rng),
            nf_pk: owner,
            nonce: NullifierNonce::random(&mut rng),
        }
    }

    pub fn commit_note(&self) -> NoteCommitment {
        self.note.commit(self.nf_pk, self.nonce)
    }

    pub fn commit_balance(&self) -> Balance {
        self.balance_blinding.commit(&self.note)
    }

    pub fn commit(&self) -> Output {
        Output {
            note_comm: self.commit_note(),
            balance: self.commit_balance(),
        }
    }
}

// as we don't have SNARKS hooked up yet, the witness will be our proof
#[derive(Debug, Clone)]
pub struct OutputProof(OutputWitness);

impl Output {
    pub fn prove(&self, w: &OutputWitness) -> Result<OutputProof, Error> {
        if &w.commit() == self {
            Ok(OutputProof(*w))
        } else {
            Err(Error::ProofFailed)
        }
    }

    pub fn verify(&self, proof: &OutputProof) -> bool {
        // verification checks the relation
        // - note_comm == commit(note || nf_pk)
        // - balance == v * hash_to_curve(Unit) + blinding * H
        let witness = &proof.0;

        self.note_comm == witness.commit_note() && self.balance == witness.commit_balance()
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(self.note_comm.as_bytes());
        bytes[32..64].copy_from_slice(&self.balance.to_bytes());
        bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{note::unit_point, nullifier::NullifierSecret};

    #[test]
    fn test_output_proof() {
        let (nmo, eth) = (unit_point("NMO"), unit_point("ETH"));
        let mut rng = rand::thread_rng();

        let witness = OutputWitness {
            note: NoteWitness::basic(10, nmo),
            balance_blinding: BalanceWitness::random(&mut rng),
            nf_pk: NullifierSecret::random(&mut rng).commit(),
            nonce: NullifierNonce::random(&mut rng),
        };

        let output = witness.commit();
        let proof = output.prove(&witness).unwrap();

        assert!(output.verify(&proof));

        let wrong_witnesses = [
            OutputWitness {
                note: NoteWitness::basic(11, nmo),
                ..witness
            },
            OutputWitness {
                note: NoteWitness::basic(10, eth),
                ..witness
            },
            OutputWitness {
                balance_blinding: BalanceWitness::random(&mut rng),
                ..witness
            },
            OutputWitness {
                nf_pk: NullifierSecret::random(&mut rng).commit(),
                ..witness
            },
            OutputWitness {
                nonce: NullifierNonce::random(&mut rng),
                ..witness
            },
        ];

        for wrong_witness in wrong_witnesses {
            assert!(output.prove(&wrong_witness).is_err());

            let wrong_output = wrong_witness.commit();
            let wrong_proof = wrong_output.prove(&wrong_witness).unwrap();
            assert!(!output.verify(&wrong_proof));
        }
    }
}
