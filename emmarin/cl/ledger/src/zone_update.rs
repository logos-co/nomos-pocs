pub use crate::error::{Error, Result};
use crate::{ledger::ProvedLedgerTransition, stf::StfProof};
use cl::zone_layer::tx::UpdateBundle;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ProvedUpdateBundle {
    pub bundle: UpdateBundle,
    pub ledger_proofs: Vec<ProvedLedgerTransition>,
    pub stf_proofs: Vec<StfProof>,
}

impl ProvedUpdateBundle {
    pub fn verify(&self) -> bool {
        let mut expected_zones = HashMap::new();
        let mut actual_zones = HashMap::new();
        for proof in &self.ledger_proofs {
            if !proof.verify() {
                return false;
            }

            for bundle in &proof.public().cross_bundles {
                expected_zones.insert(bundle.id, HashSet::from_iter(bundle.zones.clone()));
                actual_zones
                    .entry(bundle.id)
                    .or_insert_with(HashSet::new)
                    .insert(proof.public().id);
            }
        }

        for (bundle, expected) in expected_zones.iter() {
            if let Some(actual) = actual_zones.get(bundle) {
                if actual != expected {
                    panic!("{:?} | {:?}", actual, expected);
                }
            } else {
                panic!();
            }
        }

        for ((update, stf_proof), ledger_proof) in self
            .bundle
            .updates
            .iter()
            .zip(self.stf_proofs.iter())
            .zip(self.ledger_proofs.iter())
        {
            if !update.well_formed() {
                return false;
            }

            if ledger_proof.public().old_ledger != update.old.ledger
                || ledger_proof.public().ledger != update.new.ledger
            {
                return false;
            }

            if stf_proof.public.old != update.old || stf_proof.public.new != update.new {
                return false;
            }
        }

        true
    }
}
