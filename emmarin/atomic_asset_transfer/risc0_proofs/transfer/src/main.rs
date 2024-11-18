/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
use cl::merkle;
use cl::partial_tx::PtxRoot;
use goas_proof_statements::zone_funds::SpendFundsPrivate;
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let TransferPrivate {
        input,
        output,
        cm_path,
        zone_id,
    } = env::read();

    assert_eq!(input.value, output.value);
    assert_eq!(input.unit, output.unit);

    // It's impossible for an attacker to steal an incoming deposit to the zone
    // because the value, unit, nonce and pk of a deposits are hidden behind a commitment

    let nf = input.nullifier();
    let input_cm = input.note_commitment(&zone_id);
    let cm_leaf = merkle::leaf(input_cm.as_bytes());
    let cm_root = merkle::path_root(cm_leaf, &cm_path);

    let cm = output.note_commitment(node_id);
    env::commit(&TransferPublic { cm, nf, cm_root });
}
