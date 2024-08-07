/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
use cl::merkle;
use cl::partial_tx::PtxRoot;
use goas_proof_statements::zone_funds::SpendFundsPrivate;
use ledger_proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let SpendFundsPrivate {
        in_zone_funds,
        out_zone_funds,
        zone_note,
        spent_note,
        spend_event,
        spend_event_state_path,
        txs_root,
        balances_root,
    } = env::read();

    let input_root = in_zone_funds.input_root();
    let output_root = out_zone_funds.output_root();

    assert_eq!(output_root, zone_note.output_root());
    assert_eq!(output_root, spent_note.output_root());
    assert_eq!(output_root, out_zone_funds.output_root());

    // ** Assert the spent event was an output of the correct zone stf **
    // The zone state field is a merkle tree over:
    //                  root
    //              /        \
    //            io          state
    //          /   \        /     \
    //      events   txs   zoneid  balances
    // We need to check that:
    // 1) There is a valid path from the spend event to the events root
    // 2) The zone id matches the one in the current funds note state
    // 3) The witnesses for spend path, txs and balances allow to calculate the correct root
    let zone_id = in_zone_funds.input.note.state; // TODO: is there more state?
    let spend_event_leaf = merkle::leaf(&spend_event.to_bytes());
    let event_root = merkle::path_root(spend_event_leaf, &spend_event_state_path);

    assert_eq!(
        merkle::root([event_root, txs_root, zone_id, balances_root]),
        zone_note.output.note.state
    );

    // Check we return the rest of the funds back to the zone
    let change = in_zone_funds
        .input
        .note
        .value
        .checked_sub(spend_event.amount)
        .unwrap();
    assert_eq!(out_zone_funds.output.note.value, change);
    // zone funds output should have the same death constraints as the zone funds input
    assert_eq!(
        in_zone_funds.input.note.death_constraint,
        out_zone_funds.output.note.death_constraint
    );
    assert_eq!(
        in_zone_funds.input.note.unit,
        out_zone_funds.output.note.unit
    );
    // ensure zone fund sk's, blindings and nonces are propagated correctly.
    assert_eq!(
        in_zone_funds.input.nf_sk.commit(),
        out_zone_funds.output.nf_pk
    );
    assert_eq!(
        in_zone_funds.input.balance_blinding,
        out_zone_funds.output.balance_blinding
    );
    assert_eq!(
        in_zone_funds.input.evolved_nonce(),
        out_zone_funds.output.nonce,
    );
    // the state is propagated
    assert_eq!(
        in_zone_funds.input.note.state,
        out_zone_funds.output.note.state,
    );

    // check the correct amount of funds is being spent
    assert_eq!(spent_note.output.note.value, spend_event.amount);
    assert_eq!(spent_note.output.note.unit, in_zone_funds.input.note.unit);
    // check the correct recipient is being paid
    assert_eq!(spent_note.output.nf_pk, spend_event.to);

    let nf = in_zone_funds.input.nullifier();
    assert_eq!(nf, spend_event.fund_nf); // ensure this event was meant for this note.

    let ptx_root = PtxRoot(merkle::node(input_root, output_root));
    env::commit(&DeathConstraintPublic { ptx_root, nf });
}
