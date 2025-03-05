use app::{StateUpdate, ZoneData, ZoneOp};
use cl::{
    crust::Tx,
    mantle::{ledger::Ledger, zone::ZoneState},
};
use ledger_proof_statements::{
    ledger::{LedgerProofPublic, SyncLog},
    stf::StfPublic,
};
use risc0_zkvm::guest::env;

fn main() {
    let mut zone_data: ZoneData = env::read();
    let old_ledger: Ledger = env::read();
    let ledger: Ledger = env::read();
    let sync_logs: Vec<SyncLog> = env::read();
    let stf: [u8; 32] = env::read();
    let ops: Vec<ZoneOp> = env::read();
    let update_tx: StateUpdate = env::read();

    let zone_id = zone_data.zone_id;

    let old_zone_data = zone_data.commit();

    for op in &ops {
        zone_data.process_op(op);
    }

    let txs: Vec<&Tx> = ops
        .iter()
        .filter_map(|op| match op {
            ZoneOp::Swap(_) => None,
            ZoneOp::AddLiquidity { tx, .. } => Some(tx),
            ZoneOp::RemoveLiquidity { tx, .. } => Some(tx),
            ZoneOp::Ledger(tx) => Some(tx),
        })
        .chain(std::iter::once(&update_tx.tx))
        .collect();

    let outputs = txs
        .iter()
        .flat_map(|tx| tx.updates.iter().filter(|u| u.zone_id == zone_id))
        .flat_map(|u| u.outputs.iter())
        .copied()
        .collect();
    // TODO: inputs missings from ledger proof public
    let _inputs: Vec<_> = txs
        .iter()
        .flat_map(|tx| tx.updates.iter().filter(|u| u.zone_id == zone_id))
        .flat_map(|u| u.inputs.iter())
        .copied()
        .collect();

    let ledger_public = LedgerProofPublic {
        old_ledger,
        ledger,
        id: zone_id,
        sync_logs,
        outputs,
    };

    env::verify(
        ledger_validity_proof::LEDGER_ID,
        &risc0_zkvm::serde::to_vec(&ledger_public).unwrap(),
    )
    .unwrap();

    let public = StfPublic {
        old: ZoneState {
            ledger: old_ledger,
            zone_data: old_zone_data,
            stf,
        },
        new: ZoneState {
            ledger,
            zone_data: zone_data.update_and_commit(&update_tx),
            stf,
        },
    };

    env::commit(&public);
}
