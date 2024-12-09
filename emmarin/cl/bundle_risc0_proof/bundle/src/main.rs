use cl::cl::BalanceWitness;
use cl::zone_layer::notes::ZoneId;
use ledger_proof_statements::bundle::{BundlePrivate, BundlePublic, LedgerUpdate};
use risc0_zkvm::{guest::env, serde};
use std::collections::BTreeMap;

fn main() {
    let bundle_private: BundlePrivate = env::read();
    let bundle_id = bundle_private.id();

    let BundlePrivate { bundle, balances } = bundle_private;
    assert_eq!(bundle.len(), balances.len());

    let mut zone_ledger_updates: BTreeMap<ZoneId, LedgerUpdate> = BTreeMap::new();

    for (ptx_public, balance) in bundle.into_iter().zip(balances.iter()) {
        assert_eq!(ptx_public.ptx.balance, balance.commit());
        env::verify(
            nomos_cl_ptx_risc0_proof::PTX_ID,
            &serde::to_vec(&ptx_public).unwrap(),
        )
        .unwrap();

        for (input, cm_mmr) in ptx_public.ptx.inputs.iter().zip(ptx_public.cm_mmrs) {
            let zone_ledger_update = zone_ledger_updates.entry(input.zone_id).or_default();

            zone_ledger_update.nullifiers.push(input.nullifier);

            zone_ledger_update
                .cm_roots
                .extend(cm_mmr.roots.iter().map(|r| r.root));
        }

        for output in &ptx_public.ptx.outputs {
            zone_ledger_updates
                .entry(output.zone_id)
                .or_default()
                .commitments
                .push(output.note_comm);
        }
    }

    assert!(BalanceWitness::combine(balances, [0u8; 16]).is_zero());

    env::commit(&BundlePublic {
        bundle_id,
        zone_ledger_updates,
    });
}
