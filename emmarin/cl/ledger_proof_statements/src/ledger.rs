use std::collections::BTreeMap;

use crate::bundle::BundleId;
use crate::bundle::BundlePublic;
use cl::cl::{merkle, NoteCommitment};
use cl::zone_layer::{
    ledger::{Ledger, LedgerWitness},
    notes::ZoneId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerProofPublic {
    pub old_ledger: Ledger,
    pub ledger: Ledger,
    pub id: ZoneId,
    pub cross_bundles: Vec<CrossZoneBundle>,
    pub outputs: Vec<NoteCommitment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerProofPrivate {
    pub ledger: LedgerWitness,
    pub id: ZoneId,
    pub bundles: Vec<LedgerBundleWitness>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerBundleWitness {
    pub bundle: BundlePublic,
    pub cm_root_proofs: BTreeMap<[u8; 32], merkle::Path>,
    pub nf_proofs: Vec<merkle::Path>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossZoneBundle {
    pub id: BundleId,
    pub zones: Vec<ZoneId>,
}
