use cl::crust::balance::NOP_VK;
/// Input Proof
use ledger_proof_statements::covenant::{SpendingCovenantPublic, SupplyCovenantPublic};
use ledger_proof_statements::tx::TxPrivate;
use risc0_zkvm::{guest::env, serde};

fn main() {
    let TxPrivate {
        tx,
        mint_units,
        burn_units,
        spend_units,
    } = env::read();

    let tx_public = tx.commit();
    let tx_root = tx_public.root;

    assert_eq!(tx.mints.len(), mint_units.len());
    for (mint, witness) in tx.mints.iter().zip(mint_units) {
        assert_eq!(mint.unit, witness.unit()); // TODO: maybe we can skip storing the unit in the mint
        if witness.minting_covenant == NOP_VK {
            continue;
        }
        env::verify(
            witness.minting_covenant,
            &serde::to_vec(&SupplyCovenantPublic {
                amount: mint.amount,
                unit: mint.unit,
                tx_root,
            })
            .unwrap(),
        )
        .unwrap();
    }

    assert_eq!(tx.burns.len(), burn_units.len());
    for (burn, witness) in tx.burns.iter().zip(burn_units) {
        assert_eq!(burn.unit, witness.unit()); // TODO: maybe we can skip storing the unit in the burn
        if witness.burning_covenant == NOP_VK {
            continue;
        }
        env::verify(
            witness.burning_covenant,
            &serde::to_vec(&SupplyCovenantPublic {
                amount: burn.amount,
                unit: burn.unit,
                tx_root,
            })
            .unwrap(),
        )
        .unwrap();
    }

    assert_eq!(tx.inputs.len(), spend_units.len());
    for (input, witness) in tx.inputs.iter().zip(spend_units) {
        assert_eq!(input.note.unit, witness.unit()); // TODO: maybe we can skip storing the unit in the note
        if witness.spending_covenant == NOP_VK {
            continue;
        }
        env::verify(
            witness.spending_covenant,
            &serde::to_vec(&SpendingCovenantPublic {
                nf: input.nullifier(),
                tx_root,
            })
            .unwrap(),
        )
        .unwrap();
    }

    env::commit(&tx_public);
}
