use crate::ptx::PtxPublic;
use cl::cl::{bundle::BundleId, Output};
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
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerProofPrivate {
    pub ledger: LedgerWitness,
    pub id: ZoneId,
    pub bundles: Vec<Vec<PtxPublic>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossZoneBundle {
    pub id: BundleId,
    pub zones: Vec<ZoneId>,
}
