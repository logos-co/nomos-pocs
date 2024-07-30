/// Output Proof
///
/// given randomness `r` and `note=(value, unit, ...)` prove that
/// - balance = balance_commit(value, unit, r)
/// - note_cm = note_commit(note)
use risc0_zkvm::guest::env;

fn main() {
    let output: cl::OutputWitness = env::read();

    // 0 does not contribute to balance, implications of this are unclear
    // therefore out of an abundance of caution, we disallow these zero
    // valued "dummy notes".
    assert!(output.note.value > 0);

    let output_cm = output.commit();
    env::commit(&output_cm);
}
