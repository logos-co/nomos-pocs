use std::collections::BTreeMap;

use ledger_proof_statements::ledger::{LedgerBundleWitness, LedgerProofPrivate, LedgerProofPublic};

use crate::bundle::ProvedBundle;
use cl::mantle::{ledger::LedgerState, zone::ZoneId};

use hex::FromHex;

#[derive(Debug, Clone)]
pub struct ProvedLedgerTransition {
    pub risc0_receipt: risc0_zkvm::Receipt,
}

impl ProvedLedgerTransition {
    pub fn prove(ledger: &mut LedgerState, zone_id: ZoneId, bundles: Vec<ProvedBundle>) -> Self {
        let mut env = risc0_zkvm::ExecutorEnv::builder();
        let mut w_bundles = Vec::new();
        let mut nullifiers = Vec::new();
        // prepare the sparse merkle tree nullifier proofs
        for proved_bundle in &bundles {
            env.add_assumption(proved_bundle.risc0_receipt.clone());

            let bundle = proved_bundle.public();

            let zone_ledger_updates = bundle
                .updates
                .get(&zone_id)
                .expect("why are we proving this bundle for this zone if it's not involved?");

            let mut cm_root_proofs = BTreeMap::new();
            for zone_ledger_update in zone_ledger_updates {
                cm_root_proofs.extend(
                    zone_ledger_update
                        .frontier_nodes
                        .iter()
                        .map(|root| (root.root, vec![])),
                );
                nullifiers.extend(zone_ledger_update.inputs.clone());
            }

            let ledger_bundle = LedgerBundleWitness {
                bundle,
                cm_root_proofs,
            };

            w_bundles.push(ledger_bundle);
        }

        let witness = LedgerProofPrivate {
            bundles: w_bundles,
            ledger: ledger.to_witness(),
            id: zone_id,
            nf_proofs: ledger.add_nullifiers(nullifiers),
        };

        for bundle in &witness.bundles {
            let updates = bundle
                .bundle
                .updates
                .get(&zone_id)
                .expect("should have a bundle from the zone we are proofing for");

            for update in updates {
                for (cm, _data) in &update.outputs {
                    ledger.add_commitment(cm);
                }
            }

            ledger.add_bundle(bundle.bundle.root);
        }

        witness.write(&mut env);
        let env = env.build().unwrap();

        // Obtain the default prover.
        let prover = risc0_zkvm::default_prover();

        let start_t = std::time::Instant::now();

        // Proof information by proving the specified ELF binary.
        // This struct contains the receipt along with statistics about execution of the guest
        let opts = risc0_zkvm::ProverOpts::succinct();
        let prove_info = prover
            .prove_with_opts(env, risc0_images::LEDGER_ELF, &opts)
            .unwrap();

        println!(
            "STARK 'ledger' prover time: {:.2?}, user_cycles: {}, total_cycles: {}",
            start_t.elapsed(),
            prove_info.stats.user_cycles,
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
            .verify(<[u8; 32]>::from_hex(risc0_images::LEDGER_ID).unwrap())
            .is_ok()
    }
}
