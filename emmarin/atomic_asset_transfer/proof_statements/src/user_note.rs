/// The User Note encodes the logic of the atomic asset transfer
///
/// The scenario is as follows:
/// The user, let's call her Alice has 100 NMO in Zone A and she wants to move it to
/// Zone B. She wants to arrange this transfer so that both the withdrawal from Zone
/// A and the deposit to Zone B occur atomically.
///
/// The Alice will create a partial tx that looks like this:
///
///   [] -> [user note]
///
/// Thep User Note will encode the logic that orchestrates the withdrawal from zone A
/// and deposit to zone B.
///
/// The User Notes constraint requires the following statements to be satisfied
/// in order for the fee to be captured.
///
/// 1. w_tx = withdraw(amt=100 NMO, from=Alice) tx was included in Zone A.
/// 2. d_tx = deposit(amt=100 NMO, to=Alice) tx was included in Zone B.
/// 3. w_tx is included in Zone A iff d_tx is included in Zone B
///
/// Details:
/// - the withdrawal in zone A must not be a general withdrawal tx, it must be bound to the user note.
///   i.e. the user_note must be present in the ptx for the withdrawal to be valid in Zone A.
use ledger_proof_statements::constraint::ConstraintPublic;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserIntent {
    pub zone_a_meta: common::ZoneMetadata,
    pub zone_b_meta: common::ZoneMetadata,
    pub withdraw: common::Withdraw,
    pub deposit: common::Deposit,
}

impl UserIntent {
    pub fn commit(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"USER_INTENT_STATE");
        hasher.update(self.zone_a_meta.id());
        hasher.update(self.zone_b_meta.id());
        hasher.update(self.withdraw.to_bytes());
        hasher.update(self.deposit.to_bytes());
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAtomicTransfer {
    // user's note
    pub user_note: cl::PartialTxInputWitness,
    pub user_intent: UserIntent,

    // the output state notes which should have included both tx's
    pub zone_a: cl::PartialTxOutputWitness,
    pub zone_b: cl::PartialTxOutputWitness,

    // proofs of identies of the above notes
    pub zone_a_roots: common::StateRoots,
    pub zone_b_roots: common::StateRoots,

    // proof that zone_a has included this withdrawal
    pub withdraw_tx: common::IncludedTxWitness,
    // proof that zone_b has included this deposit
    pub deposit_tx: common::IncludedTxWitness,
}

impl UserAtomicTransfer {
    pub fn assert_constraints(&self) -> ConstraintPublic {
        // user committed to these actions in the user note
        assert_eq!(self.user_intent.commit(), self.user_note.input.note.state);

        // ensure we are interacting with the correct zone notes
        crate::assert_is_zone_note(
            &self.user_intent.zone_a_meta,
            &self.zone_a.output.note,
            &self.zone_a_roots,
        );
        crate::assert_is_zone_note(
            &self.user_intent.zone_b_meta,
            &self.zone_b.output.note,
            &self.zone_b_roots,
        );

        // ensure txs were included in the respective zones
        assert!(self.zone_a_roots.verify_tx_inclusion(&self.withdraw_tx));
        assert!(self.zone_b_roots.verify_tx_inclusion(&self.deposit_tx));

        // ensure the txs are the same ones the user requested
        assert_eq!(
            common::Tx::Withdraw(self.user_intent.withdraw),
            self.withdraw_tx.tx
        );
        assert_eq!(
            common::Tx::Deposit(self.user_intent.deposit),
            self.deposit_tx.tx
        );

        let input_root = self.user_note.input_root();
        let output_root = self.zone_a.output_root();
        assert_eq!(output_root, self.zone_b.output_root());

        let ptx_root = cl::PtxRoot(cl::merkle::node(input_root, output_root));
        let nf = self.user_note.input.nullifier();
        ConstraintPublic { ptx_root, nf }
    }
}
