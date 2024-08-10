use cl::{PartialTxInputWitness, PartialTxOutputWitness};
use common::{BoundTx, IncludedTxWitness, StateRoots, StateWitness, ZoneMetadata};
use goas_proof_statements::{
    user_note::{UserAtomicTransfer, UserIntent},
    zone_funds::SpendFundsPrivate,
    zone_state::ZoneStatePrivate,
};

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
    println!("STARK 'zone_stf' prover time: {:.2?}", start_t.elapsed());
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
    println!("STARK 'zone_fund' prover time: {:.2?}", start_t.elapsed());
    let receipt = prove_info.receipt;
    ledger::DeathProof::from_risc0(goas_risc0_proofs::SPEND_ZONE_FUNDS_ID, receipt)
}

pub fn prove_user_atomic_transfer(
    user_note: PartialTxInputWitness,
    user_intent: UserIntent,
    zone_a: PartialTxOutputWitness,
    zone_b: PartialTxOutputWitness,
    zone_a_roots: StateRoots,
    zone_b_roots: StateRoots,
    withdraw_tx: IncludedTxWitness,
    deposit_tx: IncludedTxWitness,
) -> ledger::DeathProof {
    let private_inputs = UserAtomicTransfer {
        user_note,
        user_intent,
        zone_a,
        zone_b,
        zone_a_roots,
        zone_b_roots,
        withdraw_tx,
        deposit_tx,
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
        .prove_with_opts(env, goas_risc0_proofs::USER_ATOMIC_TRANSFER_ELF, &opts)
        .unwrap();
    println!(
        "STARK 'user atomic transfer' prover time: {:.2?}",
        start_t.elapsed()
    );
    let receipt = prove_info.receipt;
    ledger::DeathProof::from_risc0(goas_risc0_proofs::USER_ATOMIC_TRANSFER_ID, receipt)
}
