use app::{AddLiquidity, SwapArgs, SwapOutput, ZoneData, ZONE_ID};
use cl::{
    crust::{InputWitness, Nonce, NullifierSecret, OutputWitness, TxWitness, UnitWitness},
    mantle::ledger::LedgerState,
};

fn nmo() -> UnitWitness {
    UnitWitness::nop(b"NMO")
}
fn mem() -> UnitWitness {
    UnitWitness::nop(b"MEM")
}

#[test]
fn pair_price() {
    let mut rng = rand::thread_rng();

    let mut swapvm_state = ZoneData::new();

    // initially there is no NMO/MEM pair
    assert_eq!(swapvm_state.pair_price(nmo().unit(), mem().unit()), None);

    let lp_sk = NullifierSecret::random(&mut rng);
    swapvm_state.add_liquidity(&AddLiquidity::new(
        nmo().unit(),
        10,
        mem().unit(),
        100,
        lp_sk.commit(),
        Nonce::random(&mut rng),
    ));

    // given that there is 1nmo:10mem in the pool, the price should show that we get 10 NEM for 1 NMO
    assert_eq!(
        swapvm_state.pair_price(nmo().unit(), mem().unit()),
        Some(10.0)
    );

    // switching the trade direction should flip the price as well
    assert_eq!(
        swapvm_state.pair_price(mem().unit(), nmo().unit()),
        Some(0.1)
    );

    // Due to slippage, the amount we get out is less than what the price would imply
    assert_eq!(
        swapvm_state.amount_out(nmo().unit(), mem().unit(), 1),
        Some(9) // 1 MEM slippage
    );
    assert_eq!(
        swapvm_state.amount_out(nmo().unit(), mem().unit(), 2),
        Some(18) // 2 MEM slippage
    );
    assert_eq!(
        swapvm_state.amount_out(nmo().unit(), mem().unit(), 5),
        Some(39) // 11 MEM slippage
    );
}

#[test]
fn simple_swap() {
    let mut rng = rand::thread_rng();

    let alice_sk = NullifierSecret::random(&mut rng);

    let alice_in = InputWitness {
        state: [0u8; 32],
        value: 10,
        unit_witness: nmo(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_sk: alice_sk,
    };

    let alice_out = OutputWitness {
        state: [0u8; 32],
        value: 100,
        unit: mem().unit(),
        nonce: Nonce::random(&mut rng),
        zone_id: ZONE_ID,
        nf_pk: alice_sk.commit(),
    };

    let mut ledger = LedgerState::default();

    // alice's input note is already in the ledger
    let alice_in_proof = ledger.add_commitment(&alice_in.note_commitment());

    let swap_tx = TxWitness::default()
        .add_input(alice_in, alice_in_proof)
        .add_output(alice_out, b"")
        .add_output(
            app::swap_goal_note(&mut rng),
            SwapArgs {
                output: SwapOutput::basic(mem().unit(), ZONE_ID, alice_sk.commit(), &mut rng),
                limit: 90,
            },
        );

    panic!()
    // alice ---- swap_tx ---> executor
}
