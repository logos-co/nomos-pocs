use cl::{merkle, nullifier::Nullifier, PtxRoot};
use goas_proof_statements::zone_funds::MergePrivate;
use ledger_proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let MergePrivate {
        funds_note,
        zone_note,
        merge_event,
        merge_event_state_path,
        txs_root,
        balances_root,
    } = env::read();

    let input_root = funds_note.input_root();
    let output_root = zone_note.output_root();
    let nf = Nullifier::new(funds_note.input.nf_sk, funds_note.input.nonce);
    // check the zone funds note is the one in the spend event
    assert_eq!(nf, merge_event.nf);

    // ** Assert merge spent event was an output of the correct zone stf **
    // The zone state field is a merkle tree over:
    //                  root
    //              /        \
    //            io          state
    //          /   \        /     \
    //      events   txs   zoneid  balances
    // We need to check that:
    // 1) There is a valid path from the spend event to the events root
    // 2) The zone id matches the one in the current funds note state
    // 3) The witnesses for merge path, txs and balances allow to calculate the correct root
    let zone_id = funds_note.input.note.state; // TODO: is there more state?
    let merge_event_leaf = merkle::leaf(&merge_event.to_bytes());
    let event_root = merkle::path_root(merge_event_leaf, &merge_event_state_path);

    assert_eq!(
        merkle::root([event_root, txs_root, zone_id, balances_root]),
        zone_note.output.note.state
    );
    let ptx_root = PtxRoot(merkle::node(input_root, output_root));

    env::commit(&DeathConstraintPublic { ptx_root, nf });
}
