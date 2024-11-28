use cl::cl::BalanceWitness;
/// Bundle Proof
///
/// The bundle proof demonstrates that the set of partial transactions
/// balance to zero. i.e. \sum inputs = \sum outputs.
///
/// This is done by proving knowledge of some blinding factor `r` s.t.
///     \sum outputs - \sum input = 0*G + r*H
///
/// To avoid doing costly ECC in stark, we compute only the RHS in stark.
/// The sums and equality is checked outside of stark during proof verification.
use risc0_zkvm::guest::env;

fn main() {
    let balance_private: ledger_proof_statements::balance::BalancePrivate = env::read();

    let balance_public = ledger_proof_statements::balance::BalancePublic {
        balances: Vec::from_iter(balance_private.balances.iter().map(|b| b.commit())),
    };

    assert!(BalanceWitness::combine(balance_private.balances, [0u8; 16]).is_zero());

    env::commit(&balance_public);
}
