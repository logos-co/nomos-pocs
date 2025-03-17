use cl::crust::{
    balance::NOP_COVENANT,
    tx::{BurnAmount, InputDerivedFields, MintAmount},
    TxWitness,
};
/// Input Proof
use ledger_proof_statements::covenant::{SpendingCovenantPublic, SupplyCovenantPublic};
use risc0_zkvm::{guest::env, serde};

fn main() {
    let tx: TxWitness = env::read();

    let mints = tx.mint_amounts();
    let burns = tx.burn_amounts();
    let inputs = tx.inputs_derived_fields();
    let tx_public = tx.commit(&mints, &burns, &inputs);
    let tx_root = tx_public.root;

    for (MintAmount { amount, unit, .. }, minting_covenant) in mints
        .iter()
        .zip(tx.mints.iter().map(|m| m.unit.minting_covenant))
    {
        if minting_covenant == NOP_COVENANT {
            continue;
        }
        env::verify(
            minting_covenant,
            &serde::to_vec(&SupplyCovenantPublic {
                amount: *amount,
                unit: *unit,
                tx_root,
            })
            .unwrap(),
        )
        .unwrap();
    }

    for (BurnAmount { unit, amount, .. }, burning_covenant) in burns
        .iter()
        .zip(tx.burns.iter().map(|b| b.unit.burning_covenant))
    {
        if burning_covenant == NOP_COVENANT {
            continue;
        }
        env::verify(
            burning_covenant,
            &serde::to_vec(&SupplyCovenantPublic {
                amount: *amount,
                unit: *unit,
                tx_root,
            })
            .unwrap(),
        )
        .unwrap();
    }

    for (InputDerivedFields { nf, .. }, spending_covenant) in inputs
        .iter()
        .zip(tx.inputs.iter().map(|w| w.unit_witness.spending_covenant))
    {
        if spending_covenant == NOP_COVENANT {
            continue;
        }
        env::verify(
            spending_covenant,
            &serde::to_vec(&SpendingCovenantPublic { nf: *nf, tx_root }).unwrap(),
        )
        .unwrap();
    }

    env::commit(&tx_public);
}
