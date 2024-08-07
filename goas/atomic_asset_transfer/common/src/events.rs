use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Spend(Spend),
}

impl Event {
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: add variant tag to byte encoding
        match self {
            Event::Spend(spend) => spend.to_bytes().to_vec(),
        }
    }
}

/// An event that authorizes spending zone funds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spend {
    pub amount: u64,
    /// The public key of the recipient
    pub to: cl::NullifierCommitment,
    /// The nullifier of note that is being spent, this is to avoid using the spend event to
    /// for multiple notes
    pub fund_nf: cl::Nullifier,
}

impl Spend {
    pub fn to_bytes(&self) -> [u8; 72] {
        let mut bytes = [0; 72];
        bytes[0..8].copy_from_slice(&self.amount.to_le_bytes());
        bytes[8..40].copy_from_slice(self.to.as_bytes());
        bytes[40..72].copy_from_slice(self.fund_nf.as_bytes());
        bytes
    }
}