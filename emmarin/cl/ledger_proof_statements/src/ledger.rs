use std::collections::BTreeMap;

use cl::{
    crust::{Bundle, BundleRoot, NoteCommitment},
    ds::indexed::BatchUpdateProof,
    ds::merkle,
    mantle::{
        ledger::{Ledger, LedgerWitness},
        ZoneId,
    },
};
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerProofPublic {
    pub old_ledger: Ledger,
    pub ledger: Ledger,
    pub id: ZoneId,
    pub sync_logs: Vec<SyncLog>,
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
    pub bundle: Bundle,
    pub cm_root_proofs: BTreeMap<[u8; 32], merkle::Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLog {
    pub bundle: BundleRoot,
    pub zones: Vec<ZoneId>,
}

impl LedgerProofPrivate {
    pub fn read() -> Self {
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
impl LedgerProofPrivate {
    pub fn write(&self, env: &mut risc0_zkvm::ExecutorEnvBuilder) {
        env.write(&self.ledger).unwrap();
        env.write(&self.id).unwrap();
        env.write(&self.bundles).unwrap();

        env.write(&self.nf_proofs.as_slice().len()).unwrap();
        env.write_slice(self.nf_proofs.as_slice());
    }
}
