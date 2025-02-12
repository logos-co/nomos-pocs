use crate::tx::TxPublic;
use cl::{
    crust::{tx::Bundle, BalanceWitness},
    ds::mmr::MMR,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePublic {
    pub bundle: Bundle,
    pub frontier_nodes: Vec<MMR>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePrivate {
    pub balance_witnesses: Vec<BalanceWitness>,
    pub tx_public: TxPublic,
}
