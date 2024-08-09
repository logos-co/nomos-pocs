use common::{BoundTx, StateWitness};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneStatePrivate {
    pub state: StateWitness,
    pub inputs: Vec<BoundTx>,
    pub zone_in: cl::PartialTxInputWitness,
    pub zone_out: cl::PartialTxOutputWitness,
    /// While the absence of birth constraints does not guarantee uniqueness of a note that can be used as
    /// zone funds, deposits and withdrawals make sure the funds are merged in a single note.
    /// This means that while there's nothing to prevent creation of notes with the same characteristics of zone
    /// funds, those would not be tracked by the zone state and can be ignored.
    pub funds_out: cl::PartialTxOutputWitness,
}
