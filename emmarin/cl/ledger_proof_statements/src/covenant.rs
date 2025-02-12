use cl::crust::{Nullifier, TxRoot, Unit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendingCovenantPublic {
    pub nf: Nullifier,
    pub tx_root: TxRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyCovenantPublic {
    pub amount: u64,
    pub unit: Unit,
    pub tx_root: TxRoot,
}
