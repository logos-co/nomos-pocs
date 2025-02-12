use cl::mantle::zone::ZoneState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StfPublic {
    pub old: ZoneState,
    pub new: ZoneState,
}
