use super::ledger::Ledger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoneNote {
    pub stf: Stf,
    pub state: State,
    pub ledger: Ledger,
    pub id: [u8; 32],
}

pub type Stf = [u8; 32];
pub type ZoneId = [u8; 32];
pub type State = [u8; 32];
