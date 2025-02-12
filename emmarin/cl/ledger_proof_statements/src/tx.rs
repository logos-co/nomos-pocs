use cl::{
    crust::{balance::UnitWitness, Tx, TxWitness},
    ds::mmr::MMR,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxPublic {
    pub tx: Tx,
    pub cm_mmrs: Vec<MMR>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxPrivate {
    pub tx: TxWitness,
    pub mint_units: Vec<UnitWitness>,
    pub burn_units: Vec<UnitWitness>,
    pub spend_units: Vec<UnitWitness>,
}
