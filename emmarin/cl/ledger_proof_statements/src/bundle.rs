use std::collections::{BTreeMap, BTreeSet};

use cl::{
    cl::{BalanceWitness, NoteCommitment, Nullifier},
    zone_layer::notes::ZoneId,
};
use risc0_zkvm::{
    guest::env,
    sha::rust_crypto::{Digest, Sha256},
};
use risc0_zkvm_io_derive::{Read, Write};
use serde::{Deserialize, Serialize};

use crate::ptx::PtxPublic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BundleId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Read)]
pub struct BundlePublic {
    pub bundle_id: BundleId,
    pub zone_ledger_updates: BTreeMap<ZoneId, LedgerUpdate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Read, Write)]
pub struct LedgerUpdate {
    // inputs in this bundle used the following roots in their cm membership proof.
    pub cm_roots: BTreeSet<[u8; 32]>,
    // these are the nullifiers of inputs used in this bundle.
    pub nullifiers: Nullifiers,
    // these are commitments to created notes in this bundle
    pub commitments: Commitments,
}

#[derive(Debug, Clone, PartialEq, Eq, Read, Write)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullifiers {
    data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commitments {
    data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Words {
    data: Vec<u8>,
}

impl risc0_zkvm_io::Read for Nullifiers {
    fn read() -> Self {
        let len: usize = env::read();
        let mut data = vec![0; len];
        env::read_slice(&mut data);
        Nullifiers { data }
    }
}

impl risc0_zkvm_io::Read for Commitments {
    fn read() -> Self {
        let len: usize = env::read();
        let mut data = vec![0; len];
        env::read_slice(&mut data);
        Commitments { data }
    }
}

impl risc0_zkvm_io::Read for Words {
    fn read() -> Self {
        let len: usize = env::read();
        let mut data = vec![0; len];
        env::read_slice(&mut data);
        Words { data }
    }
}

impl IntoIterator for Nullifiers {
    type Item = Nullifier;
    type IntoIter = std::vec::IntoIter<Nullifier>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
            .chunks_exact(32)
            .map(|chunk| Nullifier(chunk.try_into().unwrap()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl IntoIterator for Commitments {
    type Item = NoteCommitment;
    type IntoIter = std::vec::IntoIter<NoteCommitment>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
            .chunks_exact(32)
            .map(|chunk| NoteCommitment(chunk.try_into().unwrap()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl IntoIterator for Words {
    type Item = [u8; 32];
    type IntoIter = std::vec::IntoIter<[u8; 32]>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
            .chunks_exact(32)
            .map(|chunk| chunk.try_into().unwrap())
            .collect::<Vec<_>>()
            .into_iter()
    }
}
