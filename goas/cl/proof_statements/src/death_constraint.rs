use cl::{Nullifier, PtxRoot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeathConstraintPublic {
    pub cm_root: [u8; 32],
    pub nf: Nullifier,
    pub ptx_root: PtxRoot,
}
