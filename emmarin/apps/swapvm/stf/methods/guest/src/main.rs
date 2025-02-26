use risc0_zkvm::guest::env;

fn main() {
    let mut inputs: SwapVmPrivate = env::read();

    let zone_id = inputs.zone_data.zone_id;

    assert_eq!(inputs.zone_data.commit(), inputs.old.zone_data);

    for op in ops {
        zone_data.process_op(tx);
    }

    let txs = ops
        .iter()
        .map(|op| match op {
            ZoneOp::Swap { tx, swap, proof } => tx,
            ZoneOp::AddLiquidity { tx, .. } => tx,
            ZoneOp::RemoveLiquidity { tx, .. } => tx,
            ZoneOp::Ledger(tx) => tx,
        })
        .collect();

    let sync_logs = vec![]; // get this from outside

    let outputs = txs
        .iter()
        .flat_map(|tx| tx.outputs.clone())
        .filter(|o| o.zone_id == zone_id)
        .collect();
    let inputs = txs
        .iter()
        .flat_map(|tx| tx.inputs.clone())
        .filter(|i| i.zone_id == zone_id)
        .collect();

    let ledger_public = LedgerProofPublic {
        old_ledger: inputs.old.ledger,
        ledger: inputs.new_ledger,
        id: zone_id,
        sync_log,
        outputs,
    };

    env::verify(
        ledger_validity_proof::LEDGER_ID,
        &serde::to_vec(&ledger_public).unwrap(),
    )
    .unwrap();
}
