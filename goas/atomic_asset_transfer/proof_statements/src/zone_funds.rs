use ledger_proof_statements::{ptx::PartialTxInputPrivate, ptx::PartialTxOutputPrivate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendFundsPrivate {
    /// The note we're spending
    pub in_zone_funds: PartialTxInputPrivate,
    /// The zone note that is authorizing the spend
    pub zone_note: PartialTxOutputPrivate,
    /// The note that is being created to send the change back to the zone
    pub out_zone_funds: PartialTxOutputPrivate,
    /// The spent funds note
    pub spent_note: PartialTxOutputPrivate,
    /// The event emitted by the zone that authorizes the spend
    pub spend_event: common::events::Spend,
    /// Path to the zone output events root
    pub spend_event_state_path: Vec<cl::merkle::PathNode>,
    /// Merkle root of txs included in the zone
    pub txs_root: [u8; 32],
    /// Merkle root of balances in the zone
    pub balances_root: [u8; 32],
}
