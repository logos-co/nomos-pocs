use std::collections::BTreeMap;

use cl::Constraint;
use common::{
    mmr::MMR, AccountId, IncludedTxWitness, SignedBoundTx, StateWitness, Tx, ZoneMetadata,
};
use goas_proof_statements::{
    user_note::UserAtomicTransfer, zone_funds::SpendFundsPrivate, zone_state::ZoneStatePrivate,
};
use rand_core::CryptoRngCore;

#[derive(Debug, Clone)]
pub struct ZoneNotes {
    pub state: StateWitness,
    pub state_note: cl::OutputWitness,
    pub fund_note: cl::OutputWitness,
}

impl ZoneNotes {
    pub fn new_with_balances(
        zone_name: &str,
        balances: BTreeMap<AccountId, u64>,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        let state = StateWitness {
            balances,
            ticket_auction: Default::default(),
            included_txs: MMR::new(),
            zone_metadata: zone_metadata(zone_name),
        };
        let state_note = zone_state_utxo(&state, &mut rng);
        let fund_note = zone_fund_utxo(state.total_balance(), state.zone_metadata, &mut rng);
        Self {
            state,
            state_note,
            fund_note,
        }
    }

    pub fn state_input_witness(&self) -> cl::InputWitness {
        cl::InputWitness::public(self.state_note)
    }

    pub fn fund_input_witness(&self) -> cl::InputWitness {
        cl::InputWitness::public(self.fund_note)
    }

    pub fn run(mut self, block_height: u64, tx: Tx) -> (Self, IncludedTxWitness) {
        let (new_state, included_tx) = self.state.apply(block_height, tx);
        self.state = new_state;

        let state_in = self.state_input_witness();
        self.state_note = cl::OutputWitness::public(cl::NoteWitness {
            state: self.state.commit().0,
            nonce: state_in.evolved_nonce(b"STATE_NONCE"),
            ..state_in.note
        });

        let fund_in = self.fund_input_witness();
        self.fund_note = cl::OutputWitness::public(cl::NoteWitness {
            value: self.state.total_balance(),
            nonce: state_in.evolved_nonce(b"FUND_NONCE"),
            ..fund_in.note
        });

        (self, included_tx)
    }
}

fn zone_fund_utxo(
    value: u64,
    zone_meta: ZoneMetadata,
    mut rng: impl CryptoRngCore,
) -> cl::OutputWitness {
    cl::OutputWitness::public(cl::NoteWitness {
        value,
        unit: *common::ZONE_CL_FUNDS_UNIT,
        constraint: zone_meta.funds_constraint,
        state: zone_meta.id(),
        nonce: cl::Nonce::random(&mut rng),
    })
}

fn zone_state_utxo(zone: &StateWitness, mut rng: impl CryptoRngCore) -> cl::OutputWitness {
    cl::OutputWitness::public(cl::NoteWitness {
        value: 1,
        unit: zone.zone_metadata.unit,
        constraint: zone.zone_metadata.zone_constraint,
        state: zone.commit().0,
        nonce: cl::Nonce::random(&mut rng),
    })
}

pub fn user_atomic_transfer_constraint() -> Constraint {
    ledger::constraint::risc0_constraint(goas_risc0_proofs::USER_ATOMIC_TRANSFER_ID)
}

pub fn zone_state_constraint() -> Constraint {
    ledger::constraint::risc0_constraint(goas_risc0_proofs::ZONE_STATE_ID)
}

pub fn zone_fund_constraint() -> Constraint {
    ledger::constraint::risc0_constraint(goas_risc0_proofs::SPEND_ZONE_FUNDS_ID)
}

pub fn zone_metadata(zone_mnemonic: &str) -> ZoneMetadata {
    ZoneMetadata {
        zone_constraint: zone_state_constraint(),
        funds_constraint: zone_fund_constraint(),
        unit: cl::note::derive_unit(zone_mnemonic),
    }
}

pub fn prove_zone_stf(
    state: StateWitness,
    block_height: u64,
    inputs: Vec<(SignedBoundTx, cl::PartialTxInputWitness)>,
    zone_in: cl::PartialTxInputWitness,
    zone_out: cl::PartialTxOutputWitness,
    funds_out: cl::PartialTxOutputWitness,
) -> ledger::ConstraintProof {
    let private_inputs = ZoneStatePrivate {
        state,
        block_height,
        inputs,
        zone_in,
        zone_out,
        funds_out,
    };

    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(&private_inputs)
        .unwrap()
        .build()
        .unwrap();

    let prover = risc0_zkvm::default_prover();

    use std::time::Instant;
    let start_t = Instant::now();
    let opts = risc0_zkvm::ProverOpts::succinct();
    let prove_info = prover
        .prove_with_opts(env, goas_risc0_proofs::ZONE_STATE_ELF, &opts)
        .unwrap();
    println!(
        "STARK 'zone_stf' prover time: {:.2?}, total_cycles: {}",
        start_t.elapsed(),
        prove_info.stats.total_cycles
    );
    let receipt = prove_info.receipt;
    ledger::ConstraintProof::from_risc0(goas_risc0_proofs::ZONE_STATE_ID, receipt)
}

pub fn prove_zone_fund_constraint(
    in_zone_funds: cl::PartialTxInputWitness,
    zone_note: cl::PartialTxOutputWitness,
    out_zone_state: &StateWitness,
    block_height: u64,
) -> ledger::ConstraintProof {
    let private_inputs = SpendFundsPrivate {
        in_zone_funds,
        zone_note,
        state_roots: out_zone_state.state_roots(),
    };

    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(&private_inputs)
        .unwrap()
        .write(&block_height)
        .unwrap()
        .build()
        .unwrap();

    let prover = risc0_zkvm::default_prover();

    use std::time::Instant;
    let start_t = Instant::now();
    let opts = risc0_zkvm::ProverOpts::succinct();
    let prove_info = prover
        .prove_with_opts(env, goas_risc0_proofs::SPEND_ZONE_FUNDS_ELF, &opts)
        .unwrap();
    println!(
        "STARK 'zone_fund' prover time: {:.2?}, total_cycles: {}",
        start_t.elapsed(),
        prove_info.stats.total_cycles
    );
    let receipt = prove_info.receipt;
    ledger::ConstraintProof::from_risc0(goas_risc0_proofs::SPEND_ZONE_FUNDS_ID, receipt)
}

pub fn prove_user_atomic_transfer(
    atomic_transfer: UserAtomicTransfer,
    block_height: u64,
) -> ledger::ConstraintProof {
    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(&atomic_transfer)
        .unwrap()
        .write(&block_height)
        .unwrap()
        .build()
        .unwrap();

    let prover = risc0_zkvm::default_prover();

    use std::time::Instant;
    let start_t = Instant::now();
    let opts = risc0_zkvm::ProverOpts::succinct();
    let prove_info = prover
        .prove_with_opts(env, goas_risc0_proofs::USER_ATOMIC_TRANSFER_ELF, &opts)
        .unwrap();
    println!(
        "STARK 'user atomic transfer' prover time: {:.2?}, total_cycles: {}",
        start_t.elapsed(),
        prove_info.stats.total_cycles
    );
    let receipt = prove_info.receipt;
    ledger::ConstraintProof::from_risc0(goas_risc0_proofs::USER_ATOMIC_TRANSFER_ID, receipt)
}

#[cfg(test)]
mod tests {
    use cl::{
        note::derive_unit, BalanceWitness, Nonce, NoteWitness, OutputWitness, PartialTxWitness,
    };
    use common::{BoundTx, Deposit, Withdraw};
    use goas_proof_statements::user_note::UserIntent;
    use ledger::ConstraintProof;
    use ledger_proof_statements::constraint::ConstraintPublic;

    use super::*;

    #[test]
    pub fn test_prove_zone_stf() {
        let mut rng = rand::thread_rng();

        let mut alice = common::new_account(&mut rng);
        let alice_vk = alice.verifying_key().to_bytes();

        let zone_start =
            ZoneNotes::new_with_balances("ZONE", BTreeMap::from_iter([(alice_vk, 32)]), &mut rng);

        let bind = OutputWitness::public(NoteWitness::basic(
            32,
            *common::ZONE_CL_FUNDS_UNIT,
            &mut rng,
        ));

        let signed_withdraw = SignedBoundTx::sign(
            BoundTx {
                tx: Tx::Withdraw(Withdraw {
                    from: alice_vk,
                    amount: 10,
                }),
                bind: bind.commit_note(),
            },
            &mut alice,
        );

        let block_height = 0;

        let zone_end = zone_start
            .clone()
            .run(block_height, signed_withdraw.bound_tx.tx)
            .0;

        let ptx = PartialTxWitness {
            inputs: vec![
                cl::InputWitness::public(bind),
                zone_start.state_input_witness(),
                zone_start.fund_input_witness(),
            ],
            outputs: vec![zone_end.state_note, zone_end.fund_note],
            balance_blinding: BalanceWitness::random_blinding(&mut rng),
        };

        let txs = vec![(signed_withdraw, ptx.input_witness(0))];

        let proof = prove_zone_stf(
            zone_start.state.clone(),
            block_height,
            txs,
            ptx.input_witness(1),
            ptx.output_witness(0),
            ptx.output_witness(1),
        );

        assert!(proof.verify(ConstraintPublic {
            nf: zone_start.state_input_witness().nullifier(),
            ptx_root: ptx.commit().root(),
            block_height: 0,
        }))
    }

    #[test]
    fn test_prove_zone_fund_constraint() {
        let mut rng = rand::thread_rng();
        let zone = ZoneNotes::new_with_balances("ZONE", BTreeMap::from_iter([]), &mut rng);

        let ptx = PartialTxWitness {
            inputs: vec![zone.fund_input_witness()],
            outputs: vec![zone.state_note],
            balance_blinding: BalanceWitness::random_blinding(&mut rng),
        };

        let block_height = 0;

        let proof = prove_zone_fund_constraint(
            ptx.input_witness(0),
            ptx.output_witness(0),
            &zone.state,
            block_height,
        );

        assert!(proof.verify(ConstraintPublic {
            nf: zone.fund_input_witness().nullifier(),
            ptx_root: ptx.commit().root(),
            block_height,
        }))
    }

    #[test]
    fn test_prove_user_atomic_transfer() {
        let mut rng = rand::thread_rng();
        let block_height = 0;

        let alice = common::new_account(&mut rng);
        let alice_vk = alice.verifying_key().to_bytes();

        let zone_a =
            ZoneNotes::new_with_balances("ZONE_A", BTreeMap::from_iter([(alice_vk, 40)]), &mut rng);
        let zone_b = ZoneNotes::new_with_balances("ZONE_B", BTreeMap::new(), &mut rng);

        let user_intent = UserIntent {
            zone_a_meta: zone_a.state.zone_metadata,
            zone_b_meta: zone_b.state.zone_metadata,
            withdraw: Withdraw {
                from: alice_vk,
                amount: 32,
            },
            deposit: Deposit {
                to: alice_vk,
                amount: 32,
            },
        };
        let user_note = cl::InputWitness::public(cl::OutputWitness::public(NoteWitness {
            value: 1,
            unit: derive_unit("INTENT"),
            constraint: ConstraintProof::nop_constraint(),
            state: user_intent.commit(),
            nonce: Nonce::random(&mut rng),
        }));

        let (zone_a, withdraw_included_witnesss) =
            zone_a.run(block_height, Tx::Withdraw(user_intent.withdraw));
        let (zone_b, deposit_included_witnesss) =
            zone_b.run(block_height, Tx::Deposit(user_intent.deposit));

        let ptx = PartialTxWitness {
            inputs: vec![user_note],
            outputs: vec![zone_a.state_note, zone_b.state_note],
            balance_blinding: BalanceWitness::random_blinding(&mut rng),
        };

        let user_atomic_transfer = UserAtomicTransfer {
            user_note: ptx.input_witness(0),
            user_intent,
            zone_a: ptx.output_witness(0),
            zone_b: ptx.output_witness(1),
            zone_a_roots: zone_a.state.state_roots(),
            zone_b_roots: zone_b.state.state_roots(),
            withdraw_tx: withdraw_included_witnesss,
            deposit_tx: deposit_included_witnesss,
        };

        let proof = prove_user_atomic_transfer(user_atomic_transfer, block_height);

        assert!(proof.verify(ConstraintPublic {
            nf: user_note.nullifier(),
            ptx_root: ptx.commit().root(),
            block_height,
        }))
    }
}
