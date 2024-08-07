use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendFundsPrivate {
    /// The note we're spending
    pub in_zone_funds: cl::PartialTxInputWitness,
    /// The zone note that is authorizing the spend
    pub zone_note: cl::PartialTxOutputWitness,
    /// The note that is being created to send the change back to the zone
    pub out_zone_funds: cl::PartialTxOutputWitness,
    /// The spent funds note
    pub spent_note: cl::PartialTxOutputWitness,
    /// The event emitted by the zone that authorizes the spend
    pub spend_event: common::events::Spend,
    /// Path to the zone output events root
    pub spend_event_state_path: Vec<cl::merkle::PathNode>,
    /// Merkle root of txs included in the zone
    pub txs_root: [u8; 32],
    /// Merkle root of balances in the zone
    pub balances_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePrivate {
    // The note we're spending
    pub funds_note: cl::PartialTxInputWitness,
    // The zone note that is authorizing the spend
    pub zone_note: cl::PartialTxOutputWitness,
    // The event emitted by the zone that authorizes spending this note
    pub merge_event: common::events::Merge,
    // Path to the zone output events root
    pub merge_event_state_path: Vec<cl::merkle::PathNode>,
    // Merkle root of txs included in the zone
    pub txs_root: [u8; 32],
    // Merkle root of balances in the zone
    pub balances_root: [u8; 32],
}
