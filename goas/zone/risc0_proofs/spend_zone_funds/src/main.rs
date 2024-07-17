/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
use cl::merkle;
use cl::nullifier::{Nullifier, NullifierNonce, NullifierSecret};
use goas_proof_statements::zone_funds::SpendFundsPrivate;
use proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() {
    let SpendFundsPrivate {
        in_zone_funds,
        out_zone_funds,
        zone_note,
        spent_note,
        spend_event,
        spend_event_state_path,
    } = env::read();

    let cm_root = in_zone_funds.cm_root();
    let ptx_root = in_zone_funds.ptx_root();
    let nf = Nullifier::new(in_zone_funds.input.nf_sk, in_zone_funds.input.nonce);
    // check the zone funds note is the one in the spend event
    assert_eq!(nf, spend_event.nf);

    assert_eq!(ptx_root, zone_note.ptx_root());
    // assert the spent event was an output of the zone stf
    let spend_event_leaf = merkle::leaf(&spend_event.to_bytes());
    // TODO: zones will have some more state
    assert_eq!(
        zone_note.output.note.state,
        merkle::path_root(spend_event_leaf, &spend_event_state_path)
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
    let mut evolved_nonce = [0; 16];
    evolved_nonce[..16]
        .copy_from_slice(&Sha256::digest(&out_zone_funds.output.nonce.as_bytes())[..16]);
    assert_eq!(
        out_zone_funds.output.nonce,
        NullifierNonce::from_bytes(evolved_nonce)
    );

    assert_eq!(ptx_root, spent_note.ptx_root());

    // check the correct amount of funds is being spent
    assert_eq!(spent_note.output.note.value, spend_event.amount);
    assert_eq!(spent_note.output.note.unit, in_zone_funds.input.note.unit);
    // check the correct recipient is being paid
    assert_eq!(spent_note.output.nf_pk, spend_event.to);

    env::commit(&DeathConstraintPublic {
        cm_root,
        ptx_root,
        nf,
    });
}
