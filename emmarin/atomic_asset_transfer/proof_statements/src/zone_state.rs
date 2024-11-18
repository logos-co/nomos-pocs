use common::{SignedBoundTx, StateWitness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneStfPrivate {
    pub state: StateWitness,
    pub ledger_witness: LedgerWitness,
    pub txs: Vec<Tx>,
    pub zone_in: cl::PartialTxInputWitness,
    pub zone_out: cl::PartialTxOutputWitness,
}
