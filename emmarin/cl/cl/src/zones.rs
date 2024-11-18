use crate::{merkle, Constraint, NoteCommitment, Nullifier, PartialTx, PartialTxWitness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pact {
    pub tx: PartialTx,
    pub to: Vec<ZoneId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PactWitness {
    pub tx: PartialTxWitness,
    pub from: ZoneId,
    pub to: Vec<ZoneId>,
}

impl PactWitness {
    pub fn commit(&self) -> Pact {
        assert_eq!(self.tx.outputs.len(), self.to.len());
        let ptx = PartialTx {
            inputs: Vec::from_iter(self.tx.inputs.iter().map(|i| i.commit(&self.from))),
            outputs: Vec::from_iter(
                self.tx
                    .outputs
                    .iter()
                    .zip(&self.to)
                    .map(|(o, z)| o.commit(z)),
            ),
            balance: self.tx.balance().commit(),
        };
        Pact {
            tx: ptx,
            to: self.to.clone(),
        }
    }
}

pub struct ZoneNote {
    pub stf: Constraint,
    pub state: State,
    pub ledger: Ledger,
    pub id: [u8; 32],
}

pub type State = [u8; 32];
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    cm_root: [u8; 32],
    nf_root: [u8; 32],
}

pub type ZoneId = [u8; 32];
pub struct StateWitness;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerWitness {
    pub commitments: Vec<NoteCommitment>,
    pub nullifiers: Vec<Nullifier>,
}

const MAX_COMM: usize = 256;
const MAX_NULL: usize = 256;

impl LedgerWitness {
    pub fn commit(&self) -> Ledger {
        Ledger {
            cm_root: self.cm_root(),
            nf_root: self.nf_root(),
        }
    }

    pub fn nf_root(&self) -> [u8; 32] {
        let bytes = self
            .nullifiers
            .iter()
            .map(|i| i.as_bytes().to_vec())
            .collect::<Vec<_>>();
        merkle::root(merkle::padded_leaves::<MAX_NULL>(&bytes))
    }

    pub fn cm_root(&self) -> [u8; 32] {
        let bytes = self
            .commitments
            .iter()
            .map(|i| i.as_bytes().to_vec())
            .collect::<Vec<_>>();
        merkle::root(merkle::padded_leaves::<MAX_COMM>(&bytes))
    }

    pub fn cm_path(&self, cm: &NoteCommitment) -> Option<Vec<merkle::PathNode>> {
        let bytes = self
            .commitments
            .iter()
            .map(|i| i.as_bytes().to_vec())
            .collect::<Vec<_>>();
        let leaves = merkle::padded_leaves::<MAX_COMM>(&bytes);
        let idx = self.commitments.iter().position(|c| c == cm)?;
        Some(merkle::path(leaves, idx))
    }
}
