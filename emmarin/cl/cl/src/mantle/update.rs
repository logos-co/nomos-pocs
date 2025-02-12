use crate::mantle::ZoneState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBundle {
    pub updates: Vec<ZoneUpdate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneUpdate {
    pub old: ZoneState,
    pub new: ZoneState,
}
