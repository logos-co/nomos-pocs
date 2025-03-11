use app::{AddLiquidity, ZONE_ID};
use cl::crust::{InputWitness, Nonce, NullifierSecret, TxWitness, UnitWitness};
use host::ExecutorState;
use ledger::tx::ProvedTx;
use rand::RngCore;

fn nmo() -> UnitWitness {
    UnitWitness::nop(b"NMO")
}
fn mem() -> UnitWitness {
    UnitWitness::nop(b"MEM")
}

fn setup_executor(mut rng: impl RngCore) -> ExecutorState {
    let mut exec_state = ExecutorState::default();

    let nmo_fund = InputWitness {
        state: [0u8; 32],
        value: 1348,
        unit_witness: nmo(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: NullifierSecret::zero(),
    };

    let (mmr, mmr_proof) = exec_state
        .ledger
        .add_commitment(&nmo_fund.note_commitment());
    exec_state.set_fund_note(nmo_fund, mmr, mmr_proof);

    let mem_fund = InputWitness {
        state: [0u8; 32],
        value: 14102,
        unit_witness: mem(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: NullifierSecret::zero(),
    };
    let (mmr, mmr_proof) = exec_state
        .ledger
        .add_commitment(&mem_fund.note_commitment());
    exec_state.set_fund_note(mem_fund, mmr, mmr_proof);

    // HACK: we don't currently support liquidity notes, we directly hard code the corresponding liquidity
    // in the swapvm instead of minting pool LP tokens
    exec_state.swapvm.add_liquidity(&AddLiquidity::new(
        nmo().unit(),
        nmo_fund.value,
        mem().unit(),
        mem_fund.value,
        NullifierSecret::random(&mut rng).commit(),
        Nonce::random(&mut rng),
    ));

    exec_state
}

#[test]
fn simple_swap() {
    let mut rng = rand::thread_rng();

    // ---- setup scenario ----
    let mut exec_state = setup_executor(&mut rng);

    // setup fund notes

    let alice_sk = NullifierSecret::random(&mut rng);

    let alice_in = InputWitness {
        state: [0u8; 32],
        value: 10,
        unit_witness: nmo(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: alice_sk,
    };

    // alice's note lands in the ledger through some other executor
    let mut other_exec_ledger = exec_state.ledger.clone();
    let alice_in_proof = other_exec_ledger.add_commitment(&alice_in.note_commitment());

    // executor becomes aware of the commitment through observing a zone update
    exec_state.observe_cms([alice_in.note_commitment()]);

    // ----- end setup ----
    // Alice now has a valid 10 NMO note, she wants to swap it for 90 MEM
    // ---- begin swap ----

    let swap_goal_nonce = Nonce::random(&mut rng);
    let swap_tx = TxWitness::default()
        .add_input(alice_in, alice_in_proof)
        .add_output(
            app::swap_goal_note(swap_goal_nonce),
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

    // ensure the pair price is above the minimum realized price (90 out / 10 in = 9.0)
    assert_eq!(
        exec_state
            .swapvm
            .pair_price(nmo().unit(), mem().unit())
            .unwrap(),
        9.0
    );

    // ensure that the realized output is above the limit order
    assert!(
        exec_state
            .swapvm
            .amount_out(nmo().unit(), mem().unit(), 10)
            .unwrap()
            >= 90
    );

    panic!();
}
