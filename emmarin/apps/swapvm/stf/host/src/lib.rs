use std::collections::BTreeMap;

use app::{swap_goal_unit, SwapArgs, ZoneData};
use cl::crust::{
    BundleWitness, InputWitness, Nonce, NoteCommitment, Nullifier, NullifierSecret, Tx, TxWitness,
    Unit,
};
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
}

impl ExecutorState {
    pub fn observe_cms(&mut self, cms: impl IntoIterator<Item = NoteCommitment>) {
        for cm in cms {
            self.ledger.add_commitment(&cm);
        }
    }

    pub fn observe_nfs(&mut self, nfs: Vec<Nullifier>) {
        self.ledger.add_nullifiers(nfs);
    }

    pub fn process_tx(&mut self, tx: &Tx) {
        if tx.balance.unit_balance(swap_goal_unit().unit()).is_neg() {
            // this is a SWAP
            let (swap_cm, swap_args_bytes) =
                &tx.updates.get(&self.swapvm.zone_id).unwrap().outputs[0];
            let swap_args: SwapArgs = cl::deserialize(&swap_args_bytes);

            // verify the user proved the correct swap goal note
            assert_eq!(
                swap_cm,
                &app::swap_goal_note(swap_args.nonce).note_commitment()
            );

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
        for (cm, _) in tx
            .updates
            .get(&self.swapvm.zone_id)
            .map(|u| u.outputs.iter())
            .unwrap_or_default()
        {
            self.ledger.add_commitment(cm);
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
                InputWitness {
                    state: [0; 32],
                    value,
                    unit_witness: todo!(),
                    nonce: Nonce::from_bytes([0; 32]),
                    zone_id: self.swapvm.zone_id,
                    nf_sk: NullifierSecret([0; 16]),
                }
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
    pub fn prove(&self, prover: &dyn Prover) -> Result<Receipt> {
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
        Ok(prove_info.receipt)
    }
}
