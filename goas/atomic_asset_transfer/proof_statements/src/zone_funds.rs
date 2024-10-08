use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendFundsPrivate {
    /// The note we're spending
    pub in_zone_funds: cl::PartialTxInputWitness,
    /// The zone note that is authorizing the spend
    pub zone_note: cl::PartialTxOutputWitness,
    /// Proof of identity of the above note
    pub state_roots: common::StateRoots,
}
