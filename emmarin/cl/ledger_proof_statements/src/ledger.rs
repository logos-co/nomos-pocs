use crate::bundle::BundlePublic;
use crate::pact::PactPublic;
use crate::ptx::PtxPublic;
use cl::cl::Output;
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
    pub cross_in: Vec<Output>,
    pub cross_out: Vec<Output>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerProofPrivate {
    pub ledger: LedgerWitness,
    pub id: ZoneId,
    pub txs: Vec<ZoneTx>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ZoneTx {
    LocalTx {
        ptxs: Vec<PtxPublic>,
        bundle: BundlePublic,
    },
    Pact(PactPublic),
}
