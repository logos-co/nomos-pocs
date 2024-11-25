use cl::{
    cl::{merkle, PartialTx, PartialTxWitness},
    zone_layer::notes::ZoneId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtxPublic {
    pub ptx: PartialTx,
    pub cm_roots: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtxPrivate {
    pub ptx: PartialTxWitness,
    pub input_cm_paths: Vec<Vec<merkle::PathNode>>,
    pub cm_roots: Vec<[u8; 32]>,
    pub from: Vec<ZoneId>,
    pub to: Vec<ZoneId>,
}
