/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
///
/// More formally, statement says:
/// for public input `nf` (nullifier) and `root_cm` (root of merkle tree over commitment set).
/// the prover has knowledge of `output = (note, nf_pk, nonce)`, `nf` and `path` s.t. that the following constraints hold
/// 0. nf_pk = hash(nf_sk)
/// 1. nf = hash(nonce||nf_sk)
/// 2. note_cm = output_commitment(output)
/// 3. verify_merkle_path(note_cm, root, path)
use cl::merkle;
use cl::nullifier::{Nullifier, NullifierNonce, NullifierSecret};
use cl::partial_tx::PtxRoot;
use proof_statements::zone_funds::{WithdrawPrivate, WithdrawPublic};
use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() {
    let WithdrawPrivate {
        in_zone_funds,
        out_zone_funds,
        zone_note,
        spent_note,
        spend_event,
        in_zone_funds_ptx_path,
        in_zone_funds_cm_path,
        out_zone_funds_ptx_path,
        zone_note_ptx_path,
        spent_note_ptx_path,
        spend_event_state_path,
        in_zone_funds_nf_sk: _,
    } = env::read();

    let in_zone_funds_cm = in_zone_funds.commit();
    let in_zone_funds_leaf = merkle::leaf(&in_zone_funds_cm.to_bytes());
    let cm_root = merkle::path_root(in_zone_funds_leaf, &in_zone_funds_cm_path);
    let ptx_root = merkle::path_root(in_zone_funds_leaf, &in_zone_funds_ptx_path);
    let in_zone_funds_nf = Nullifier::new(in_zone_funds.nf_sk, in_zone_funds.nonce);
    // check the zone funds note is the one in the spend event
    assert_eq!(in_zone_funds_nf, spend_event.nf);

    let zone_note_cm = zone_note.commit_note();
    let zone_note_leaf = merkle::leaf(zone_note_cm.as_bytes());
    assert_eq!(
        ptx_root,
        merkle::path_root(zone_note_leaf, &zone_note_ptx_path)
    );

    // assert the spent event was an output of the zone stf
    let spend_event_leaf = merkle::leaf(&spend_event.to_bytes());
    // TODO: zones will have some more state
    assert_eq!(
        zone_note.note.state,
        merkle::path_root(spend_event_leaf, &spend_event_state_path)
    );

    let out_zone_funds_cm = out_zone_funds.commit_note();
    let out_zone_funds_leaf = merkle::leaf(out_zone_funds_cm.as_bytes());
    assert_eq!(
        ptx_root,
        merkle::path_root(out_zone_funds_leaf, &out_zone_funds_ptx_path)
    );

    // Check we return the rest of the funds back to the zone
    let change = in_zone_funds
        .note
        .balance
        .value
        .checked_sub(spend_event.amount)
        .unwrap();
    assert_eq!(out_zone_funds.note.balance.value, change);
    // zone funds output should have the same death constraints as the zone funds input
    assert_eq!(
        out_zone_funds.note.death_constraint,
        in_zone_funds.note.death_constraint
    );
    assert_eq!(
        out_zone_funds.note.balance.unit,
        in_zone_funds.note.balance.unit
    );
    // zone funds nullifier, nonce and value blinding should be public so that everybody can spend it
    assert_eq!(
        out_zone_funds.nf_pk,
        NullifierSecret::from_bytes([0; 16]).commit()
    );
    assert_eq!(
        out_zone_funds.note.balance.blinding,
        in_zone_funds.note.balance.blinding
    );
    let mut evolved_nonce = [0; 16];
    evolved_nonce[..16].copy_from_slice(&Sha256::digest(&out_zone_funds.nonce.as_bytes())[..16]);
    assert_eq!(
        out_zone_funds.nonce,
        NullifierNonce::from_bytes(evolved_nonce)
    );

    let spent_note_cm = spent_note.commit_note();
    let spent_note_leaf = merkle::leaf(spent_note_cm.as_bytes());
    assert_eq!(
        ptx_root,
        merkle::path_root(spent_note_leaf, &spent_note_ptx_path)
    );

    // check the correct amount of funds is being spent
    assert_eq!(spent_note.note.balance.value, spend_event.amount);
    assert_eq!(
        spent_note.note.balance.unit,
        in_zone_funds.note.balance.unit
    );
    // check the correct recipient is being paid
    assert_eq!(spent_note.nf_pk, spend_event.to);

    env::commit(&WithdrawPublic {
        cm_root,
        ptx_root: PtxRoot::from(ptx_root),
        nf: in_zone_funds_nf,
    });
}
