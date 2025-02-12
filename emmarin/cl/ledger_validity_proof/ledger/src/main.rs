use cl::merkle;
use ledger_proof_statements::ledger::{
    LedgerBundleWitness, LedgerProofPrivate, LedgerProofPublic, SyncLog,
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
    let mut sync_logs = vec![];
    let mut outputs = vec![];

    let mut nullifiers = vec![];

    for LedgerBundleWitness {
        bundle,
        cm_root_proofs,
    } in bundles
    {
        env::verify(
            nomos_cl_bundle_risc0_proof::BUNDLE_ID,
            &serde::to_vec(&bundle).unwrap(),
        )
        .unwrap();

        // TODO: do not add local updates
        sync_logs.push(SyncLog {
            id: bundle.root,
            zones: bundle.updates.iter().map(|update| update.zone_id).collect(),
        });

        if let Some(ledger_update) = bundle
            .updates
            .into_iter()
            .filter(|update| update.zone_id == id)
            .next()
        {
            for frontier in &ledger_update.frontier_nodes {
                for node in &frontier.roots {
                    let past_cm_root_proof = cm_root_proofs
                        .get(&node.root)
                        .expect("missing cm root proof");
                    let expected_current_cm_root = merkle::path_root(node.root, past_cm_root_proof);
                    assert!(old_ledger.valid_cm_root(expected_current_cm_root))
                }
            }

            for cm in &ledger_update.outputs {
                ledger.add_commitment(cm);
                outputs.push(*cm)
            }

            nullifiers.extend(ledger_update.inputs);
        }
    }

    // TODO: sort outside and check
    nullifiers.sort();
    ledger.assert_nfs_update(&nullifiers, &nf_proofs);

    env::commit(&LedgerProofPublic {
        old_ledger: old_ledger.commit(),
        ledger: ledger.commit(),
        id,
        sync_logs,
        outputs,
    });
}
