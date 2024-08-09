use common::{BoundTx, StateWitness};
use goas_proof_statements::{zone_funds::SpendFundsPrivate, zone_state::ZoneStatePrivate};

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
