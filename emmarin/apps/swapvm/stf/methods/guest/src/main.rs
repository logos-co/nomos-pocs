use app::{StateUpdate, ZoneData, ZoneOp, SwapArgs};
use cl::{
    crust::{BundleWitness, TxRoot},
    mantle::{
        ledger::{Ledger, LedgerWitness},
        zone::ZoneState,
    },
};
use ledger_proof_statements::{
    stf::StfPublic,
};
use risc0_zkvm::guest::env;

fn main() {
    let mut zone_data: ZoneData = env::read();
    let mut ledger_witness: LedgerWitness = env::read();
    let stf: [u8; 32] = env::read();
    let new_ledger: Ledger = env::read();
    let bundles: Vec<BundleWitness> = env::read();
    let ops: Vec<ZoneOp> = env::read();
    let update_tx: StateUpdate = env::read();

    let zone_id = zone_data.zone_id;

    let old_state = ZoneState {
        ledger: ledger_witness.commit(),
        zone_data: zone_data.commit(),
        stf,
    };

    for op in &ops {
        zone_data.process_op(op);
    }

    for bundle in bundles {
        ledger_witness.add_bundle(bundle.root());

        for tx in bundle.txs {
            let Some(zone_update) = tx.updates.get(&zone_id) else {
                // this tx does not concern this zone, ignore it.
                continue
            };

            zone_data.validate_no_pools(zone_update);

            if tx.balance.unit_balance(app::swap_goal_unit().unit()).is_neg() {
                // This TX encodes a SWAP request.
                // as a simplifying assumption, we will assume that the SWAP goal note is the only output
                assert_eq!(zone_update.outputs.len(), 1);
                let (swap_goal_cm, swap_args_bytes) = &zone_update.outputs[0];
                let swap_args: SwapArgs = cl::deserialize(&swap_args_bytes);

                // ensure the witness corresponds to the swap goal cm
                assert_eq!(
                    swap_goal_cm,
                    &app::swap_goal_note(swap_args.nonce).note_commitment()
                );
                panic!("zone_data.swap()");
            }
        }
    }

    // ensure we had processed all the ops
    assert!(ops.is_empty());

    // ensure that we've seen every bundle in this ledger update
    assert_eq!(ledger_witness.bundles.commit(), new_ledger.bundles_root);

    let public = StfPublic {
        old: old_state,
        new: ZoneState {
            ledger: new_ledger,
            zone_data: zone_data.update_and_commit(&update_tx),
            stf,
        },
    };

    env::commit(&public);
}
