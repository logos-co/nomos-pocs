use std::collections::BTreeMap;

use common::{BoundTx, StateWitness, Tx, ZoneMetadata};
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
        balances: BTreeMap<u32, u64>,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        let state = StateWitness {
            balances,
            included_txs: vec![],
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

    pub fn run(mut self, txs: impl IntoIterator<Item = Tx>) -> Self {
        for tx in txs {
            self.state = self.state.apply(tx);
        }

        let state_in = self.state_input_witness();
        self.state_note = cl::OutputWitness::public(
            cl::NoteWitness {
                state: self.state.commit().0,
                ..state_in.note
            },
            state_in.evolved_nonce(b"STATE_NONCE"),
        );

        let fund_in = self.fund_input_witness();
        self.fund_note = cl::OutputWitness::public(
            cl::NoteWitness {
                value: self.state.total_balance(),
                ..fund_in.note
            },
            state_in.evolved_nonce(b"FUND_NONCE"),
        );
        self
    }
}

fn zone_fund_utxo(
    value: u64,
    zone_meta: ZoneMetadata,
    mut rng: impl CryptoRngCore,
) -> cl::OutputWitness {
    cl::OutputWitness::public(
        cl::NoteWitness {
            value,
            unit: *common::ZONE_CL_FUNDS_UNIT,
            death_constraint: zone_meta.funds_vk,
            state: zone_meta.id(),
        },
        cl::NullifierNonce::random(&mut rng),
    )
}

fn zone_state_utxo(zone: &StateWitness, mut rng: impl CryptoRngCore) -> cl::OutputWitness {
    cl::OutputWitness::public(
        cl::NoteWitness {
            value: 1,
            unit: zone.zone_metadata.unit,
            death_constraint: zone.zone_metadata.zone_vk,
            state: zone.commit().0,
        },
        cl::NullifierNonce::random(&mut rng),
    )
}

pub fn user_atomic_transfer_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(
        goas_risc0_proofs::USER_ATOMIC_TRANSFER_ID,
    )
}

pub fn zone_state_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(goas_risc0_proofs::ZONE_STATE_ID)
}

pub fn zone_fund_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(
        goas_risc0_proofs::SPEND_ZONE_FUNDS_ID,
    )
}

pub fn zone_metadata(zone_mnemonic: &str) -> ZoneMetadata {
    ZoneMetadata {
        zone_vk: zone_state_death_constraint(),
        funds_vk: zone_fund_death_constraint(),
        unit: cl::note::unit_point(zone_mnemonic),
    }
}

pub fn prove_zone_stf(
    state: StateWitness,
    inputs: Vec<BoundTx>,
    zone_in: cl::PartialTxInputWitness,
    zone_out: cl::PartialTxOutputWitness,
    funds_out: cl::PartialTxOutputWitness,
) -> ledger::DeathProof {
    let private_inputs = ZoneStatePrivate {
        state,
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
    ledger::DeathProof::from_risc0(goas_risc0_proofs::ZONE_STATE_ID, receipt)
}

pub fn prove_zone_fund_withdraw(
    in_zone_funds: cl::PartialTxInputWitness,
    zone_note: cl::PartialTxOutputWitness,
    out_zone_state: &StateWitness,
) -> ledger::DeathProof {
    let private_inputs = SpendFundsPrivate {
        in_zone_funds,
        zone_note,
        state_witness: out_zone_state.clone(),
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
        .prove_with_opts(env, goas_risc0_proofs::SPEND_ZONE_FUNDS_ELF, &opts)
        .unwrap();
    println!(
        "STARK 'zone_fund' prover time: {:.2?}, total_cycles: {}",
        start_t.elapsed(),
        prove_info.stats.total_cycles
    );
    let receipt = prove_info.receipt;
    ledger::DeathProof::from_risc0(goas_risc0_proofs::SPEND_ZONE_FUNDS_ID, receipt)
}

pub fn prove_user_atomic_transfer(atomic_transfer: UserAtomicTransfer) -> ledger::DeathProof {
    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(&atomic_transfer)
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
    ledger::DeathProof::from_risc0(goas_risc0_proofs::USER_ATOMIC_TRANSFER_ID, receipt)
}
