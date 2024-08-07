use common::{Input, StateWitness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneStatePrivate {
    pub state: StateWitness,
    pub inputs: Vec<Input>,
    pub zone_in: cl::PartialTxInputWitness,
    pub zone_out: cl::PartialTxOutputWitness,
}
