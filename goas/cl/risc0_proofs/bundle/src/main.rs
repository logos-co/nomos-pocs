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
    let zero_witness: cl::BalanceWitness = env::read();
    let zero_balance = cl::Balance::zero(zero_witness);
    env::commit(&zero_balance);
}
