/// The User Note encodes the logic of the atomic asset transfer
///
/// The scenario is as follows:
/// The user, let's call her Alice has 100 NMO in Zone A and she wants to move it to
/// Zone B. She wants to arrange this transfer so that both the withdrawal from Zone
/// A and the deposit to Zone B occur atomically.
///
/// The Alice will create a partial tx that looks like this:
///
///     [fee note] -> [user note]
///
/// The User Note will encode the logic that orchestrates the withdrawal from zone A
/// and deposit to zone B.
///
/// The User Notes death constraint requires the following statements to be satisfied
/// in order for the fee to be captured.
/// 
/// 1. w_tx = withdraw(amt=100 NMO, from=Alice) tx was included in Zone A.
/// 2. d_tx = deposit(amt=100 NMO, to=Alice) tx was included in Zone B.
/// 3. w_tx is included in Zone A iff d_tx is included in Zone B
///
/// Details:
/// - the withdrawal in zone A must not be a general withdrawal tx, it must be bound to the user note.
///   i.e. the user_note must be present in the ptx for the withdrawal to be valid in Zone A.
