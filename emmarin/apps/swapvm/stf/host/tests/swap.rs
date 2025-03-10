use app::{AddLiquidity, ZoneData, ZONE_ID};
use cl::{
    crust::{InputWitness, Nonce, NullifierSecret, TxWitness, UnitWitness},
    mantle::ledger::LedgerState,
};

fn nmo() -> UnitWitness {
    UnitWitness::nop(b"NMO")
}
fn mem() -> UnitWitness {
    UnitWitness::nop(b"MEM")
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

    let mut ledger = LedgerState::default();

    // alice's input note is already in the ledger
    let alice_in_proof = ledger.add_commitment(&alice_in.note_commitment());

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

    let swap_tx_proof = ledger::tx::ProvedTx::prove(swap_tx, vec![], vec![]).unwrap();

    // alice ---- (swap_tx, swap_tx_proof) ---> executor

    let mut swapvm_state = ZoneData::new();

    swapvm_state.add_liquidity(&AddLiquidity::new(
        nmo().unit(),
        1348,
        mem().unit(),
        14102,
        NullifierSecret::random(&mut rng).commit(),
        Nonce::random(&mut rng),
    ));

    // ensure the pair price is above the minimum realized price (90 out / 10 in = 9.0)
    assert_eq!(
        swapvm_state.pair_price(nmo().unit(), mem().unit()).unwrap(),
        9.0
    );

    // ensure that the realized output is above the limit order
    assert!(
        swapvm_state
            .amount_out(nmo().unit(), mem().unit(), 10)
            .unwrap()
            >= 90
    );

    panic!();
}
