use super::notes::ZoneNote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBundle {
    pub updates: Vec<ZoneUpdate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneUpdate {
    pub old: ZoneNote,
    pub new: ZoneNote,
}

impl ZoneUpdate {
    pub fn new(old: ZoneNote, new: ZoneNote) -> Self {
        assert_eq!(old.id, new.id);
        Self { old, new }
    }

    pub fn well_formed(&self) -> bool {
        self.old.id == self.new.id
    }
}
