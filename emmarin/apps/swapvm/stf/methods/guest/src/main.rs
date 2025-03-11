use app::{SwapArgs, ZoneData};
use cl::{
    crust::{BundleWitness, InputWitness},
    mantle::{
        ledger::{Ledger, LedgerWitness},
        zone::ZoneState,
    },
};
use ledger_proof_statements::stf::StfPublic;
use risc0_zkvm::guest::env;

fn main() {
    let mut zone_data: ZoneData = env::read();
    let mut ledger_witness: LedgerWitness = env::read();
    let new_ledger: Ledger = env::read();
    let stf: [u8; 32] = env::read();
    let mut bundle: BundleWitness = env::read();
    let pools_notes: Vec<InputWitness> = env::read();

    let zone_id = zone_data.zone_id;

    let old_state = ZoneState {
        ledger: ledger_witness.commit(),
        zone_data: zone_data.commit(),
        stf,
    };

    // The last bundle should be a single executor tx that updates the zone data
    let executor_tx = bundle.txs.pop().unwrap();

    ledger_witness.add_bundle(bundle.root());

    for tx in bundle.txs {
        let Some(zone_update) = tx.updates.get(&zone_id) else {
            // this tx does not concern this zone, ignore it.
            continue;
        };

        assert!(zone_data.validate_no_pools(zone_update));

        // is it a SWAP?
        if tx
            .balance
            .unit_balance(app::swap_goal_unit().unit())
            .is_neg()
        {
            // This TX encodes a SWAP request.
            // as a simplifying assumption, we will assume that the SWAP goal note is the only output
            // and a single input represents the funds provided by the user for the swap.
            assert_eq!(zone_update.outputs.len(), 1);
            assert_eq!(zone_update.inputs.len(), 1);
            let (swap_goal_cm, swap_args_bytes) = &zone_update.outputs[0];
            let swap_args: SwapArgs = cl::deserialize(&swap_args_bytes);

            // ensure the witness corresponds to the swap goal cm
            assert_eq!(
                swap_goal_cm,
                &app::swap_goal_note(swap_args.nonce).note_commitment()
            );

            let funds = tx
                .balance
                .balances
                .iter()
                .find(|bal| bal.unit != app::swap_goal_unit().unit())
                .unwrap();
            let t_in = funds.unit;
            let amount_in = funds.pos - funds.neg;
            zone_data.swap(t_in, amount_in, swap_args);
        }
    }

    // ensure that we've seen every bundle in this ledger update
    assert_eq!(ledger_witness.bundles.commit(), new_ledger.bundles_root);

    let public = StfPublic {
        old: old_state,
        new: ZoneState {
            ledger: new_ledger,
            zone_data: zone_data.update_and_commit(&executor_tx, &pools_notes),
            stf,
        },
    };

    env::commit(&public);
}
