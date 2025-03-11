use app::ZoneData;
use cl::crust::{NullifierSecret, UnitWitness};

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

    swapvm_state.add_liquidity(nmo().unit(), mem().unit(), 10, 100);

    // given that there is 1nmo:10mem in the pool, the price should show that we get 10 MEM for 1 NMO
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
        Some(16) // 4 MEM slippage
    );
    assert_eq!(
        swapvm_state.amount_out(nmo().unit(), mem().unit(), 5),
        Some(33) // 17 MEM slippage
    );
}
