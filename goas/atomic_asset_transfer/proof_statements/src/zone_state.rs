use common::{Input, StateWitness};
use ledger_proof_statements::{ptx::PartialTxInputPrivate, ptx::PartialTxOutputPrivate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneStatePrivate {
    pub state: StateWitness,
    pub inputs: Vec<Input>,
    pub zone_in: PartialTxInputPrivate,
    pub zone_out: PartialTxOutputPrivate,
}
