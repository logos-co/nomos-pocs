use std::collections::BTreeMap;

use app::{swap_goal_unit, SwapArgs, ZoneData};
use cl::crust::{BundleWitness, InputWitness, NoteCommitment, Nullifier, Tx, TxWitness, Unit};
use cl::ds::mmr::{MMRFolds, MMRProof, MMR};
use cl::mantle::ledger::Ledger;
use cl::mantle::ledger::LedgerState;
use cl::mantle::ZoneState;
use ledger::stf::{risc0_stf, StfProof};
use ledger_proof_statements::ledger::SyncLog;
use methods::{STF_ELF, STF_ID};
use risc0_zkvm::{ExecutorEnv, Prover, Result};

#[derive(Debug)]
struct FundNote {
    note: InputWitness,
    mmr: MMR,
    path: MMRProof,
}

impl FundNote {
    fn evolve(&self, new_amount: u64) -> InputWitness {
        let mut new_note = self.note.clone();
        new_note.value = new_amount;
        new_note.nonce = new_note.evolved_nonce(b"SWAP");
        new_note
    }
}

#[derive(Debug, Default)]
pub struct ExecutorState {
    pub ledger: LedgerState,
    pub swapvm: ZoneData,
    fund_notes: BTreeMap<Unit, FundNote>,
    goal_notes: Vec<(InputWitness, MMR, MMRProof)>,
}

impl ExecutorState {
    pub fn zone_state(&self) -> ZoneState {
        ZoneState {
            stf: risc0_stf(STF_ID),
            zone_data: self.swapvm.commit(),
            ledger: self.ledger.to_witness().commit(),
        }
    }

    pub fn observe_cm(&mut self, cm: &NoteCommitment) -> ((MMR, MMRProof), MMRFolds) {
        let folds = self.ledger.commitments.folds(&cm.0);

        for (_, fund_note) in self.fund_notes.iter_mut() {
            assert_eq!(fund_note.mmr, self.ledger.commitments);
            fund_note
                .path
                .update(&fund_note.note.note_commitment().0, &folds);
        }

        let proof = self.ledger.add_commitment(cm);

        for (_, fund_note) in self.fund_notes.iter_mut() {
            fund_note.mmr = self.ledger.commitments.clone();
        }

        (proof, folds)
    }

    pub fn observe_nfs(&mut self, nfs: Vec<Nullifier>) {
        self.ledger.add_nullifiers(nfs);
    }

    pub fn process_tx(&mut self, tx: &Tx) {
        let Some(swapvm_update) = tx.updates.get(&self.swapvm.zone_id) else {
            // this tx is not related to the swapvm zone
            return;
        };

        let mut output_mmr_proofs = BTreeMap::<NoteCommitment, MMRProof>::new();

        for (cm, _) in &swapvm_update.outputs {
            let (proof, folds) = self.observe_cm(cm);

            for (other_cm, other_cm_proof) in output_mmr_proofs.iter_mut() {
                other_cm_proof.update(&other_cm.0, &folds);
            }

            output_mmr_proofs.insert(*cm, proof.1);

            for (other_cm, other_cm_proof) in &output_mmr_proofs {
                assert!(self
                    .ledger
                    .commitments
                    .verify_proof(&other_cm.0, &other_cm_proof))
            }
        }

        for nf in &swapvm_update.inputs {
            self.ledger.add_nullifiers(vec![*nf]);
        }

        if tx.balance.unit_balance(swap_goal_unit().unit()).is_neg() {
            // this is a SWAP
            let (swap_goal_cm, swap_args_bytes) = &swapvm_update.outputs[0];
            let swap_args: SwapArgs = cl::deserialize(&swap_args_bytes);

            // verify the user proved the correct swap goal note
            let swap_goal_witness = app::swap_goal_note(swap_args.nonce);
            assert_eq!(swap_goal_cm, &swap_goal_witness.note_commitment());

            self.goal_notes.push((
                swap_goal_witness,
                self.ledger.commitments.clone(),
                output_mmr_proofs[swap_goal_cm].clone(),
            ));

            // assume there are only the goal unit and tokenIn units at play
            assert_eq!(tx.balance.balances.len(), 2);

            let balance_in = tx
                .balance
                .balances
                .iter()
                .find(|bal| bal.unit != swap_goal_unit().unit())
                .unwrap();

            let token_in = balance_in.unit;
            assert_eq!(balance_in.neg, 0);
            assert!(balance_in.pos > 0);

            let amount_in = balance_in.pos;
            self.swapvm.swap(token_in, amount_in, swap_args);
        }
    }

    pub fn update_and_get_executor_tx(&mut self) -> (TxWitness, Vec<InputWitness>) {
        let mut tx = TxWitness::default();
        let mut new_fund_notes = Vec::new();
        let mut ledger = self.ledger.clone();

        let expected_pool_balances = self.swapvm.expected_pool_balances();
        let fund_notes = std::mem::take(&mut self.fund_notes);

        for note in &self.swapvm.swaps_output {
            tx = tx.add_output(note.clone(), "");
            ledger.add_commitment(&note.note_commitment());
        }

        for (unit, value) in expected_pool_balances {
            let note = if let Some(note) = fund_notes.get(&unit) {
                note.evolve(value)
            } else {
                panic!("dynamically created fund notes are not supported");
            };
            new_fund_notes.push(note);
            let output = note.to_output();
            tx = tx.add_output(output, "");
            let (mmr, path) = ledger.add_commitment(&output.note_commitment());
            self.fund_notes
                .insert(note.unit_witness.unit(), FundNote { note, mmr, path });
        }

        for (_, FundNote { note, mmr, path }) in fund_notes.into_iter() {
            tx = tx.add_input(note, (mmr, path));
        }

        for (goal_note, mmr, path) in std::mem::take(&mut self.goal_notes) {
            tx = tx.add_input(goal_note, (mmr, path));
        }

        (tx, new_fund_notes)
    }

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
    pub fund_notes: Vec<InputWitness>,
    pub bundle: BundleWitness,
}

impl StfPrivate {
    pub fn prove(&self, prover: &dyn Prover) -> Result<StfProof> {
        let env = ExecutorEnv::builder()
            .write(&self.zone_data)?
            .write(&self.old_ledger)?
            .write(&self.new_ledger)?
            .write(&STF_ID)?
            .write(&self.bundle)?
            .write(&self.fund_notes)?
            .build()?;

        let prove_info = prover.prove(env, STF_ELF)?;

        debug_assert!(prove_info.receipt.verify(STF_ID).is_ok());

        Ok(StfProof::from_risc0(risc0_stf(STF_ID), prove_info.receipt))
    }
}
