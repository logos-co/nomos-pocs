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
    } = env::read();

    let old_ledger = ledger.clone();
    let mut cross_bundles = vec![];
    let mut outputs = vec![];

    for LedgerBundleWitness {
        bundle,
        cm_root_proofs,
        nf_proofs,
    } in bundles
    {
        env::verify(
            nomos_cl_bundle_risc0_proof::BUNDLE_ID,
            &serde::to_vec(&bundle).unwrap(),
        )
        .unwrap();

        if let Some(ledger_update) = bundle.zone_ledger_updates.get(&id) {
            for past_cm_root in &ledger_update.cm_roots {
                let past_cm_root_proof = cm_root_proofs
                    .get(past_cm_root)
                    .expect("missing cm root proof");
                let expected_current_cm_root = merkle::path_root(*past_cm_root, past_cm_root_proof);
                assert!(old_ledger.valid_cm_root(expected_current_cm_root))
            }

            let mut sorted_nullifiers = ledger_update.nullifiers.clone();
            // TODO: sort outside and check
            sorted_nullifiers.sort();
            // TODO: remove nullifier duplication
            assert_eq!(sorted_nullifiers, nf_proofs.nullifiers());
            ledger.assert_nfs_update(&nf_proofs);

            for cm in &ledger_update.commitments {
                ledger.add_commitment(cm);
                outputs.push(*cm)
            }
        }

        cross_bundles.push(CrossZoneBundle {
            id: bundle.bundle_id,
            zones: bundle.zone_ledger_updates.into_keys().collect(),
        });
    }

    env::commit(&LedgerProofPublic {
        old_ledger: old_ledger.commit(),
        ledger: ledger.commit(),
        id,
        cross_bundles,
        outputs,
    });
}
