/// Output Proof
///
/// given randomness `r` and `note=(value, unit, ...)` prove that
/// - balance = balance_commit(value, unit, r)
/// - note_cm = note_commit(note)
use risc0_zkvm::guest::env;

fn main() {
    let output: cl::OutputWitness = env::read();
    let output_cm = output.commit();
    env::commit(&output_cm);
}
