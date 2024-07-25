use cl::{merkle, partial_tx::MAX_OUTPUTS, PtxRoot};
use common::*;
use goas_proof_statements::zone_funds::Spend;
use proof_statements::death_constraint::DeathConstraintPublic;
use risc0_zkvm::guest::env;

fn withdraw(mut state: StateWitness, withdraw: Withdraw) -> StateWitness {
    state.included_txs.push(Input::Withdraw(withdraw));

    let Withdraw {
        from,
        amount,
        to,
        nf,
    } = withdraw;

    let from_balance = state.balances.entry(from).or_insert(0);
    *from_balance = from
        .checked_sub(amount)
        .expect("insufficient funds in account");
    let spend_auth = Spend {
        amount: amount.into(),
        to,
        nf,
    };

    state.output_events.push(Event::Spend(spend_auth));
    state
}

fn deposit(
    mut state: StateWitness,
    deposit: Deposit,
    pub_inputs: DeathConstraintPublic,
) -> StateWitness {
    // check the note witness was indeed included in this transaction,
    // can be spent by the zone (has the correct death constraints) and is of the
    // expected unit

    let input = &deposit.deposit.input.note;

    assert_eq!(
        deposit.deposit.ptx_root(),
        pub_inputs.ptx_root,
        "deposit note not included in ptx"
    );
    // the deposit note can't be already spendable by the zone
    assert_ne!(input.death_constraint, ZONE_FUNDS_VK);

    let amount = input.balance.value as u32;
    let to = AccountId::from_be_bytes(<[u8; 4]>::try_from(&input.state[0..4]).unwrap());

    let to_balance = state.balances.entry(to).or_insert(0);
    *to_balance = to_balance
        .checked_add(amount)
        .expect("overflow when depositing");

    // we also check there's no other output in the tx besides the zone note and the zone funds to avoid
    // redirecting the deposit note value outside of the zone
    let zone_note = &deposit.zone_note;
    assert_eq!(zone_note.note.balance.unit, *ZONE_UNIT);
    assert_eq!(zone_note.nf_pk, ZONE_NF_PK);
    // TODO: should we check it's *this* note?
    let zone_funds = &deposit.zone_funds;
    assert_eq!(zone_funds.nf_pk, ZONE_NF_PK);
    assert_eq!(zone_funds.note.balance.unit, *ZONE_CL_FUNDS_UNIT);
    assert_eq!(zone_funds.note.death_constraint, ZONE_FUNDS_VK);

    let leaves = merkle::padded_leaves::<MAX_OUTPUTS>(&[
        zone_note.commit().to_bytes().into(),
        zone_funds.commit().to_bytes().into(),
    ]);
    assert_eq!(
        PtxRoot(merkle::root(leaves)),
        pub_inputs.ptx_root,
        "unexpected tx output, only zone funds and note allowed"
    );

    state.included_txs.push(Input::Deposit(deposit));
    state
}

fn main() {
    let public_inputs: DeathConstraintPublic = env::read();
    let inputs: Vec<Input> = env::read();
    let mut state: StateWitness = env::read();

    for input in inputs {
        match input {
            Input::Withdraw(input) => state = withdraw(state, input),
            Input::Deposit(input) => state = deposit(state, input, public_inputs),
        }
    }

    env::commit(&state.commit());
}
