use cl::cl::{
    mmr::{MMRProof, MMR},
    PartialTx, PartialTxWitness,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtxPublic {
    pub ptx: PartialTx,
    pub cm_mmr: Vec<MMR>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtxPrivate {
    pub ptx: PartialTxWitness,
    pub input_cm_proofs: Vec<(MMR, MMRProof)>,
}
