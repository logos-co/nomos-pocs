use std::collections::{BTreeMap, BTreeSet};

use cl::{
    cl::{BalanceWitness, NoteCommitment, Nullifier},
    zone_layer::notes::ZoneId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ptx::PtxPublic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BundleId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePublic {
    pub bundle_id: BundleId,
    pub zone_ledger_updates: BTreeMap<ZoneId, LedgerUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LedgerUpdate {
    // inputs in this bundle used the following roots in their cm membership proof.
    pub cm_roots: BTreeSet<[u8; 32]>,
    // these are the nullifiers of inputs used in this bundle.
    pub nullifiers: Vec<Nullifier>,
    // these are commitments to created notes in this bundle
    pub commitments: Vec<NoteCommitment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePrivate {
    pub bundle: Vec<PtxPublic>,
    pub balances: Vec<BalanceWitness>,
}

impl BundlePrivate {
    pub fn id(&self) -> BundleId {
        // TODO: change to merkle root
        let mut hasher = Sha256::new();
        hasher.update(b"NOMOS_CL_BUNDLE_ID");
        for ptx in &self.bundle {
            hasher.update(ptx.ptx.root().0);
        }

        BundleId(hasher.finalize().into())
    }
}
