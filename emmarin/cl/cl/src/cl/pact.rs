use crate::zone_layer::ZoneId;
use crate::cl::{PartialTx, PartialTxWitness};
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
