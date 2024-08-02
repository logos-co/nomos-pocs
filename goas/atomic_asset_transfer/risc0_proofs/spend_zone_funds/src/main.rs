/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
use cl::merkle;
use cl::nullifier::{Nullifier, NullifierSecret};
use goas_proof_statements::zone_funds::SpendFundsPrivate;
use proof_statements::death_constraint::DeathConstraintPublic;
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

    let ptx_root = in_zone_funds.ptx_root();
    let nf = Nullifier::new(in_zone_funds.input.nf_sk, in_zone_funds.input.nonce);
    // check the zone funds note is the one in the spend event
    assert_eq!(nf, spend_event.nf);
    assert_eq!(ptx_root, zone_note.ptx_root());

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

    assert_eq!(ptx_root, out_zone_funds.ptx_root());

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
        out_zone_funds.output.note.death_constraint,
        in_zone_funds.input.note.death_constraint
    );
    assert_eq!(
        out_zone_funds.output.note.unit,
        in_zone_funds.input.note.unit
    );
    // zone funds nullifier, nonce and value blinding should be public so that everybody can spend it
    assert_eq!(
        out_zone_funds.output.nf_pk,
        NullifierSecret::from_bytes([0; 16]).commit()
    );
    assert_eq!(
        out_zone_funds.output.balance_blinding,
        in_zone_funds.input.balance_blinding
    );
    assert_eq!(
        out_zone_funds.output.nonce,
        in_zone_funds.input.evolved_nonce()
    );
    // the state is propagated
    assert_eq!(
        out_zone_funds.output.note.state,
        in_zone_funds.input.note.state
    );

    assert_eq!(ptx_root, spent_note.ptx_root());

    // check the correct amount of funds is being spent
    assert_eq!(spent_note.output.note.value, spend_event.amount);
    assert_eq!(spent_note.output.note.unit, in_zone_funds.input.note.unit);
    // check the correct recipient is being paid
    assert_eq!(spent_note.output.nf_pk, spend_event.to);

    env::commit(&DeathConstraintPublic { ptx_root, nf });
}
