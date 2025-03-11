use std::collections::BTreeMap;

use app::{swap_goal_unit, StateUpdate, ZoneData, ZoneOp};
use cl::crust::{BundleWitness, InputWitness, NoteCommitment, Nullifier, Unit};
use cl::ds::mmr::{MMRProof, MMR};
use cl::mantle::ledger::Ledger;
use cl::mantle::ledger::LedgerState;
use ledger_proof_statements::ledger::SyncLog;
use methods::{STF_ELF, STF_ID};
use risc0_zkvm::{ExecutorEnv, Prover, Receipt, Result};

#[derive(Debug)]
struct FundNote {
    note: InputWitness,
    mmr: MMR,
    path: MMRProof,
}

#[derive(Debug, Default)]
pub struct ExecutorState {
    pub ledger: LedgerState,
    pub swapvm: ZoneData,
    pub fund_notes: BTreeMap<Unit, FundNote>,
}

impl ExecutorState {
    pub fn observe_cms(&mut self, cms: impl IntoIterator<Item = NoteCommitment>) {
        for cm in cms {
            self.ledger.add_commitment(&cm);

            // update merkle proofs for each fund note.
            for (_, fund_note) in self.fund_notes.iter_mut() {
                let folds = fund_note.mmr.folds(&cm.0);
                fund_note
                    .path
                    .update(&fund_note.note.note_commitment().0, folds);
            }
        }
    }

    pub fn observe_nfs(&mut self, nfs: Vec<Nullifier>) {
        self.ledger.add_nullifiers(nfs);
    }

    // pub fn bundle_tx(&mut self, proved_tx: ProvedTx) -> StfPrivate {
    //     let tx = proved_tx.public();

    //     if tx.balance.unit_balance(swap_goal_unit()).is_neg() {
    //         // this is a SWAP
    //         let (swap_cm, swap_args_bytes) = &tx.outputs[0];
    //         let swap_args: SwapArgs = cl::deserialize(&swap_args_bytes);

    //         // verify the user proved the correct swap goal note
    //         assert_eq!(
    //             swap_cm,
    //             &app::swap_goal_note(swap_args.nonce).note_commitment()
    //         );

    //         // assume there are only the goal unit and tokenIn units at play
    //         assert_eq!(tx.balance.balances.len(), 2);

    //         let balance_in = tx
    //             .balance
    //             .balances
    //             .iter()
    //             .find(|bal| bal.unit != swap_goal_unit())
    //             .unwrap();

    //         let token_in = balance_in.unit;
    //         assert_eq!(balance_in.neg, 0);
    //         assert!(balance_in.pos > 0);

    //         let amount_in = balance_in.pos;
    //         let amount_out = self
    //             .swapvm
    //             .amount_out(token_in, swap_args.output.unit, amount_in)
    //             .unwrap();

    //         // ensure we can satisfy the limit order
    //         assert!(amount_out > swap_args.limit);

    //         // now build the balancing tx

    //         let balancing_tx = TxWitness::default()
    //             .add_input(self.fund_notes[token_in], self.fund_notes[token_in].1)
    //             .add_input(self.fund_notes[token_out]);
    //     }
    // }

    pub fn set_fund_note(&mut self, note: InputWitness, mmr: MMR, path: MMRProof) {
        self.fund_notes
            .insert(note.unit_witness.unit(), FundNote { note, mmr, path });
    }
}

pub struct StfPrivate {
    pub zone_data: ZoneData,
    pub old_ledger: Ledger,
    pub new_ledger: Ledger,
    pub sync_logs: Vec<SyncLog>,
    pub ops: Vec<ZoneOp>,
    pub update_tx: StateUpdate,
    pub bundles: BundleWitness,
}

impl StfPrivate {
    pub fn prove(&self, prover: &impl Prover) -> Result<Receipt> {
        let env = ExecutorEnv::builder()
            .write(&self.zone_data)?
            .write(&self.old_ledger)?
            .write(&self.new_ledger)?
            .write(&self.sync_logs)?
            .write(&STF_ID)?
            .write(&self.ops)?
            .write(&self.update_tx)?
            .build()?;

        let prove_info = prover.prove(env, STF_ELF)?;

        debug_assert!(prove_info.receipt.verify(STF_ID).is_ok());
        Ok(prove_info.receipt)
    }
}
