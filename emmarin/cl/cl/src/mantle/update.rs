use crate::mantle::ZoneState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchUpdate {
    pub updates: Vec<Update>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Update {
    pub old: ZoneState,
    pub new: ZoneState,
}
