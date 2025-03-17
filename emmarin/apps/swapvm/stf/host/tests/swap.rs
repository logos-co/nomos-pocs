use app::ZONE_ID;
use cl::crust::{BundleWitness, InputWitness, Nonce, NullifierSecret, TxWitness, UnitWitness};
use cl::mantle::ledger::LedgerState;
use cl::mantle::update::{BatchUpdate, Update};
use host::{ExecutorState, StfPrivate};
use ledger::ledger::ProvedLedgerTransition;
use ledger::update::ProvedBatchUpdate;
use ledger::{bundle::ProvedBundle, tx::ProvedTx};
use rand::RngCore;

fn nmo() -> UnitWitness {
    UnitWitness::nop(b"NMO")
}
fn mem() -> UnitWitness {
    UnitWitness::nop(b"MEM")
}

fn setup_executor(mut rng: impl RngCore, ledger: LedgerState) -> ExecutorState {
    let mut exec_state = ExecutorState::default();
    exec_state.ledger = ledger;

    let nmo_fund = InputWitness {
        state: [0u8; 32],
        value: 1348,
        unit_witness: nmo(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: NullifierSecret::zero(),
    };

    let ((mmr, mmr_proof), _) = exec_state.observe_cm(&nmo_fund.note_commitment());
    exec_state.set_fund_note(nmo_fund, mmr, mmr_proof);

    let mem_fund = InputWitness {
        state: [0u8; 32],
        value: 14102,
        unit_witness: mem(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: NullifierSecret::zero(),
    };
    let ((mmr, mmr_proof), _) = exec_state.observe_cm(&mem_fund.note_commitment());
    exec_state.set_fund_note(mem_fund, mmr, mmr_proof);

    // HACK: we don't currently support liquidity notes, we directly hard code the corresponding liquidity
    // in the swapvm instead of minting pool LP tokens
    exec_state
        .swapvm
        .add_liquidity(nmo().unit(), mem().unit(), nmo_fund.value, mem_fund.value);

    exec_state
}

#[test]
fn simple_swap() {
    let mut rng = rand::thread_rng();
    let ledger = LedgerState::default();

    // ---- setup scenario ----

    let alice_sk = NullifierSecret::random(&mut rng);

    let alice_in = InputWitness {
        state: [0u8; 32],
        value: 10,
        unit_witness: nmo(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: alice_sk,
    };

    let mut exec_state = setup_executor(&mut rng, ledger);
    let (alice_in_proof, _) = exec_state.observe_cm(&alice_in.note_commitment());

    // ----- end setup ----
    // Alice now has a valid 10 NMO note, she wants to swap it for 90 MEM
    // ---- begin swap ----

    let old_zone_state = exec_state.zone_state();
    let old_zone_data = exec_state.swapvm.clone();

    let mut temp_ledger_state = exec_state.ledger.clone();

    let swap_goal_nonce = Nonce::random(&mut rng);
    let swap_tx = TxWitness::default()
        .add_input(alice_in, alice_in_proof)
        .add_output(
            app::swap_goal_note(swap_goal_nonce).to_output(),
            app::SwapArgs {
                output: app::SwapOutput::basic(mem().unit(), ZONE_ID, alice_sk.commit(), &mut rng),
                limit: 90,
                nonce: swap_goal_nonce,
            },
        );

    let swap_tx_proof = ProvedTx::prove(swap_tx, vec![], vec![]).unwrap();

    //
    // alice ---- (swap_tx, swap_tx_proof) ---> executor
    //
    // alice sends the tx to an executor
    exec_state.process_tx(&swap_tx_proof.public());

    // the executor builds the solving tx
    let (exec_tx, fund_notes) = exec_state.update_and_get_executor_tx();
    let proved_exec_tx = ProvedTx::prove(exec_tx, vec![], vec![]).unwrap();

    let swap_bundle = BundleWitness {
        txs: vec![swap_tx_proof.public(), proved_exec_tx.public()],
    };

    let swap_bundle_proof = ProvedBundle::prove(vec![swap_tx_proof, proved_exec_tx]);
    exec_state.ledger.add_bundle(swap_bundle.root());
    exec_state.observe_nfs(
        swap_bundle
            .clone()
            .commit()
            .updates
            .get(&exec_state.swapvm.zone_id)
            .unwrap()
            .into_iter()
            .flat_map(|u| u.inputs.iter().copied())
            .collect::<Vec<_>>(),
    );
    // prove stf
    let stf_proof = StfPrivate {
        zone_data: old_zone_data,
        old_ledger: temp_ledger_state.to_witness(),
        new_ledger: exec_state.ledger.clone().to_witness().commit(),
        fund_notes,
        bundle: swap_bundle,
    }
    .prove(risc0_zkvm::default_prover().as_ref())
    .unwrap();

    let ledger_proof =
        ProvedLedgerTransition::prove(&mut temp_ledger_state, ZONE_ID, vec![swap_bundle_proof]);

    let new_zone_state = exec_state.zone_state();

    assert_eq!(ledger_proof.public().old_ledger, old_zone_state.ledger);
    assert_eq!(ledger_proof.public().ledger, new_zone_state.ledger);

    let zone_update = ProvedBatchUpdate {
        batch: BatchUpdate {
            updates: vec![Update {
                old: old_zone_state,
                new: new_zone_state,
            }],
        },
        ledger_proofs: vec![ledger_proof],
        stf_proofs: vec![stf_proof],
    };

    assert!(zone_update.verify())
}
