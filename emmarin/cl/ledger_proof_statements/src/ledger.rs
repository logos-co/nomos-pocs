use std::collections::BTreeMap;

use crate::bundle::BundleId;
use crate::bundle::BundlePublic;
use cl::cl::{indexed::BatchUpdateProof, merkle, NoteCommitment};
use cl::zone_layer::{
    ledger::{Ledger, LedgerWitness},
    notes::ZoneId,
};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerProofPublic {
    pub old_ledger: Ledger,
    pub ledger: Ledger,
    pub id: ZoneId,
    pub cross_bundles: Vec<CrossZoneBundle>,
    pub outputs: Vec<NoteCommitment>,
}

#[derive(Debug, Clone)]
pub struct LedgerProofPrivate {
    pub ledger: LedgerWitness,
    pub id: ZoneId,
    pub bundles: Vec<LedgerBundleWitness>,
    pub nf_proofs: BatchUpdateProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerBundleWitness {
    pub bundle: BundlePublic,
    pub cm_root_proofs: BTreeMap<[u8; 32], merkle::Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossZoneBundle {
    pub id: BundleId,
    pub zones: Vec<ZoneId>,
}

impl crate::io::Read for LedgerProofPrivate {
    fn read() -> Self {
        let ledger = env::read();
        let id = env::read();
        let bundles = env::read();
        let nf_proofs_len: usize = env::read();
        let mut data = vec![0; nf_proofs_len];
        env::read_slice(&mut data);

        LedgerProofPrivate {
            ledger,
            id,
            bundles,
            nf_proofs: BatchUpdateProof::from_raw_data(data),
        }
    }
}

#[cfg(not(target_os = "zkvm"))]
impl crate::io::Write for LedgerProofPrivate {
    fn write(&self, env: &mut risc0_zkvm::ExecutorEnvBuilder) {
        env.write(&self.ledger).unwrap();
        env.write(&self.id).unwrap();
        env.write(&self.bundles).unwrap();

        env.write(&self.nf_proofs.as_slice().len()).unwrap();
        env.write_slice(self.nf_proofs.as_slice());
    }
}
