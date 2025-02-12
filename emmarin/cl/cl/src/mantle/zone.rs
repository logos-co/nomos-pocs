use super::ledger::Ledger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoneState {
    pub stf: Stf,
    pub zone_data: ZoneData,
    pub ledger: Ledger,
}

pub type Stf = [u8; 32];
pub type ZoneId = [u8; 32];
pub type ZoneData = [u8; 32];
