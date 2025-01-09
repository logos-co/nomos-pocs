use cl::cl::merkle;
use ledger_proof_statements::ledger::{
    CrossZoneBundle, LedgerBundleWitness, LedgerProofPrivate, LedgerProofPublic,
};
use risc0_zkvm::{guest::env, serde};

fn main() {
    let LedgerProofPrivate {
        mut ledger,
        id,
        bundles,
        nf_proofs,
    } = LedgerProofPrivate::read();
    let old_ledger = ledger.clone();
    let mut cross_bundles = vec![];
    let mut outputs = vec![];

    let mut nullifiers = vec![];

    for LedgerBundleWitness {
        mut bundle,
        cm_root_proofs,
    } in bundles
    {
        env::verify(
            nomos_cl_bundle_risc0_proof::BUNDLE_ID,
            &serde::to_vec(&bundle).unwrap(),
        )
        .unwrap();

        // TODO: do not add local updates
        cross_bundles.push(CrossZoneBundle {
            id: bundle.bundle_id,
            zones: bundle.zone_ledger_updates.keys().copied().collect(),
        });

        if let Some(ledger_update) = bundle.zone_ledger_updates.remove(&id) {
            for past_cm_root in &ledger_update.cm_roots {
                let past_cm_root_proof = cm_root_proofs
                    .get(past_cm_root)
                    .expect("missing cm root proof");
                let expected_current_cm_root = merkle::path_root(*past_cm_root, past_cm_root_proof);
                assert!(old_ledger.valid_cm_root(expected_current_cm_root))
            }

            for cm in &ledger_update.commitments {
                ledger.add_commitment(cm);
                outputs.push(*cm)
            }

            nullifiers.extend(ledger_update.nullifiers);
        }
    }

    // TODO: sort outside and check
    nullifiers.sort();
    ledger.assert_nfs_update(&nullifiers, &nf_proofs);

    env::commit(&LedgerProofPublic {
        old_ledger: old_ledger.commit(),
        ledger: ledger.commit(),
        id,
        cross_bundles,
        outputs,
    });
}
