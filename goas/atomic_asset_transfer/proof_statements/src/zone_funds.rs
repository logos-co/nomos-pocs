use proof_statements::{ptx::PartialTxInputPrivate, ptx::PartialTxOutputPrivate};
use serde::{Deserialize, Serialize};

/// An event that authorizes spending zone funds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spend {
    pub amount: u64,
    /// The public key of the recipient
    pub to: cl::NullifierCommitment,
    /// The nullifier of note that is being spent, this is to avoid using the spend event to
    /// for multiple notes
    pub nf: cl::Nullifier,
}

impl Spend {
    pub fn to_bytes(&self) -> [u8; 72] {
        let mut bytes = [0; 72];
        bytes[0..8].copy_from_slice(&self.amount.to_le_bytes());
        bytes[8..40].copy_from_slice(self.to.as_bytes());
        bytes[40..72].copy_from_slice(self.nf.as_bytes());
        bytes
    }
}

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
    pub spend_event: Spend,
    /// Path to the zone output state
    pub spend_event_state_path: Vec<cl::merkle::PathNode>,
}
