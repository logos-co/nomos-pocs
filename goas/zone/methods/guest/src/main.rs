use common::*;
use proof_statements::zone_funds::Spend;
use risc0_zkvm::guest::env;

fn withdraw(mut state: StateWitness, withdraw: Withdraw) -> StateWitness {
    state.included_txs.push(Input::Withdraw(withdraw));

    let Withdraw {
        from,
        amount,
        to,
        nf,
    } = withdraw;

    let from = state.balances.entry(from).or_insert(0);
    *from = from.checked_sub(amount).unwrap();
    let spend_auth = Spend {
        amount: amount.into(),
        to,
        nf,
    };

    state.output_events.push(Event::Spend(spend_auth));
    state
}

fn main() {
    let inputs: Vec<Input> = env::read();
    let mut state: StateWitness = env::read();

    for input in inputs {
        match input {
            Input::Withdraw(input) => {
                state = withdraw(state, input);
            }
        }
    }

    env::commit(&state.commit());
}
