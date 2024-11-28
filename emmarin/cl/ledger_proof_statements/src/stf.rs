use cl::zone_layer::notes::ZoneNote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StfPublic {
    pub old: ZoneNote,
    pub new: ZoneNote,
}
