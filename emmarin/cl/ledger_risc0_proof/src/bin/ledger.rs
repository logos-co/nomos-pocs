use cl::ds::merkle;
use hex::FromHex;
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
            <[u8; 32]>::from_hex(risc0_images::BUNDLE_ID).unwrap(),
            &serde::to_vec(&bundle).unwrap(),
        )
        .unwrap();

        if bundle.updates.len() > 1 {
            // This is a cross zone bundle, add a sync log for it to ensure all zones
            // also approve it.
            sync_logs.push(SyncLog {
                bundle: bundle.root,
                zones: bundle.updates.keys().copied().collect(),
            });
        }

        let ledger_updates = bundle
            .updates
            .get(&id)
            .expect("attempting to prove a bundle that is not for this zone");

        for ledger_update in ledger_updates {
            for node in &ledger_update.frontier_nodes {
                let past_cm_root_proof = cm_root_proofs
                    .get(&node.root)
                    .expect("missing cm root proof");

                let expected_current_cm_root = merkle::path_root(node.root, past_cm_root_proof);
                assert!(
                    old_ledger.valid_cm_root(expected_current_cm_root)
                        || ledger.valid_cm_root(expected_current_cm_root)
                );
            }

            for (cm, _data) in &ledger_update.outputs {
                ledger.add_commitment(cm);
                outputs.push(*cm);
            }
            nullifiers.extend(ledger_update.inputs.clone());
        }

        ledger.add_bundle(bundle.root);
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
