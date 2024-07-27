use cl::{Nullifier, PtxRoot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeathConstraintPublic {
    pub nf: Nullifier,
    pub ptx_root: PtxRoot,
}
