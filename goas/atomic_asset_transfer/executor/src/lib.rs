use common::{events::Event, Input, StateWitness};
use goas_proof_statements::{zone_funds::SpendFundsPrivate, zone_state::ZoneStatePrivate};
use ledger_proof_statements::ptx::{PartialTxInputPrivate, PartialTxOutputPrivate};

pub fn prove_zone_stf(
    state: StateWitness,
    inputs: Vec<Input>,
    zone_in: PartialTxInputPrivate,
    zone_out: PartialTxOutputPrivate,
) -> ledger::DeathProof {
    let private_inputs = ZoneStatePrivate {
        state,
        inputs,
        zone_in,
        zone_out,
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
    in_zone_funds: PartialTxInputPrivate,
    zone_note: PartialTxOutputPrivate,
    out_zone_funds: PartialTxOutputPrivate,
    spent_note: PartialTxOutputPrivate,
    out_zone_state: &StateWitness,
    withdraw: common::Withdraw,
) -> ledger::DeathProof {
    let spend_event = withdraw.to_event();
    let private_inputs = SpendFundsPrivate {
        in_zone_funds,
        zone_note,
        out_zone_funds,
        spent_note,
        spend_event,
        spend_event_state_path: out_zone_state.event_merkle_path(Event::Spend(spend_event)),
        balances_root: out_zone_state.balances_root(),
        txs_root: out_zone_state.included_txs_root(),
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
