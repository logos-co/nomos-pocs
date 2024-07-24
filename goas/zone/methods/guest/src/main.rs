use cl::{merkle, PtxRoot};
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
    let leaf = merkle::leaf(
        deposit
            .deposit_note
            .commit(deposit.nf_pk, deposit.nonce)
            .as_bytes(),
    );
    assert_eq!(
        PtxRoot(merkle::path_root(leaf, &deposit.ptx_path)),
        pub_inputs.ptx_root,
        "deposit note not included in ptx"
    );
    assert_eq!(deposit.deposit_note.death_constraint, ZONE_FUNDS_VK);
    assert_eq!(deposit.deposit_note.balance.unit, *ZONE_CL_FUNDS_UNIT);

    let amount = deposit.deposit_note.balance.value as u32;
    let to =
        AccountId::from_be_bytes(<[u8; 4]>::try_from(&deposit.deposit_note.state[0..4]).unwrap());

    let to_balance = state.balances.entry(to).or_insert(0);
    *to_balance = to_balance
        .checked_add(amount)
        .expect("overflow when depositing");

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
