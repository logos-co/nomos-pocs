/// Zone Funds Spend Proof
///
/// Our goal: prove the zone authorized spending of funds
use cl::merkle;
use cl::partial_tx::PtxRoot;
use goas_proof_statements::zone_funds::SpendFundsPrivate;
use ledger_proof_statements::constraint::ConstraintPublic;
use risc0_zkvm::guest::env;

fn main() {
    let SpendFundsPrivate {
        in_zone_funds,
        zone_note,
        state_roots,
    } = env::read();

    let input_root = in_zone_funds.input_root();
    let output_root = zone_note.output_root();

    let ptx_root = PtxRoot(merkle::node(input_root, output_root));

    // 1) Check the zone note is the correct one
    assert_eq!(in_zone_funds.input.note.state, state_roots.zone_id);
    assert_eq!(zone_note.output.note.state, state_roots.commit().0);

    let nf = in_zone_funds.input.nullifier();

    env::commit(&ConstraintPublic { ptx_root, nf });
}
