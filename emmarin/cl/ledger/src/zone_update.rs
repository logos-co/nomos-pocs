pub use crate::error::{Error, Result};
use crate::{ledger::ProvedLedgerTransition, stf::StfProof};
use cl::zone_layer::tx::UpdateBundle;
use std::collections::HashSet;

pub struct ProvedUpdateBundle {
    pub bundle: UpdateBundle,
    pub ledger_proofs: Vec<ProvedLedgerTransition>,
    pub stf_proofs: Vec<StfProof>,
}

impl ProvedUpdateBundle {
    pub fn verify(&self) -> bool {
        let mut consumed_commitments = HashSet::new();
        let mut produced_commitments = HashSet::new();
        for proof in &self.ledger_proofs {
            if !proof.verify() {
                return false;
            }

            for comm in &proof.public.cross_out {
                if produced_commitments.insert(comm) {
                    // already in?
                }
            }
            for comm in &proof.public.cross_in {
                if consumed_commitments.insert(comm) {
                    // already in?
                }
            }
        }

        // check that cross zone transactions match
        if consumed_commitments != produced_commitments {
            return false;
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

            if ledger_proof.public.old_ledger != update.old.ledger
                || ledger_proof.public.ledger != update.new.ledger
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
