/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
use cl::merkle;
use cl::partial_tx::PtxRoot;
use goas_proof_statements::zone_funds::SpendFundsPrivate;
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let WithdrawPrivate {
        input,
        cm_path,
        zone_id,
        authorized_pks,
    } = env::read();

    assert!(authorized_pks.contains(&input.nf_sk.commit()));

    let nf = input.nullifier();
    let input_cm = input.note_commitment(&zone_id);
    let cm_leaf = merkle::leaf(input_cm.as_bytes());
    let cm_root = merkle::path_root(cm_leaf, &cm_path);

    env::commit(&WithdrawPublic { nf, cm_root });
}
