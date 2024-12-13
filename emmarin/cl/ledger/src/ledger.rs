use std::collections::BTreeMap;

use ledger_proof_statements::ledger::{
    CompactNullifierProofs, LedgerBundleWitness, LedgerProofPrivate, LedgerProofPublic,
};

use crate::bundle::ProvedBundle;
use cl::zone_layer::{ledger::LedgerState, notes::ZoneId};

#[derive(Debug, Clone)]
pub struct ProvedLedgerTransition {
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedLedgerTransition {
    pub fn prove(mut ledger: LedgerState, zone_id: ZoneId, bundles: Vec<ProvedBundle>) -> Self {
        let mut witness = LedgerProofPrivate {
            bundles: Vec::new(),
            ledger: ledger.to_witness(),
            id: zone_id,
        };

        let mut env = risc0_zkvm::ExecutorEnv::builder();

        // prepare the sparse merkle tree nullifier proofs
        for proved_bundle in &bundles {
            env.add_assumption(proved_bundle.risc0_receipt.clone());

            let bundle = proved_bundle.public();

            let zone_ledger_update = bundle
                .zone_ledger_updates
                .get(&zone_id)
                .expect("why are we proving this bundle for this zone if it's not involved?");

            let cm_root_proofs =
                BTreeMap::from_iter(zone_ledger_update.cm_roots.iter().map(|root| {
                    // We make the simplifying assumption that bundle proofs
                    // are done w.r.t. the latest MMR (hence, empty merkle proofs)
                    //
                    // We can remove this assumption by tracking old MMR roots in the LedgerState
                    (*root, vec![])
                }));

            let mut nf_proofs = Vec::new();
            for nf in &zone_ledger_update.nullifiers {
                let nf_proof = ledger.add_nullifier(*nf);
                nf_proofs.push(nf_proof);
            }

            nf_proofs.reverse();

            let ledger_bundle = LedgerBundleWitness {
                bundle,
                cm_root_proofs,
                nf_proofs: CompactNullifierProofs::from_paths(nf_proofs),
            };

            witness.bundles.push(ledger_bundle)
        }

        let env = env.write(&witness).unwrap().build().unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, ledger_validity_proof::LEDGER_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'ledger' prover time: {:.2?}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.total_cycles
        );

        Self {
            risc0_receipt: prove_info.receipt,
        }
    }

    pub fn public(&self) -> LedgerProofPublic {
        self.risc0_receipt
            .journal
            .decode::<LedgerProofPublic>()
            .unwrap()
    }

    pub fn verify(&self) -> bool {
        self.risc0_receipt
            .verify(ledger_validity_proof::LEDGER_ID)
            .is_ok()
    }
}
