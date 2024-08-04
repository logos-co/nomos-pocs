use std::collections::BTreeMap;

use cl::{BalanceWitness, NoteWitness, NullifierNonce, NullifierSecret};
use common::{events::Event, Input, StateWitness, ZoneMetadata, ZONE_CL_FUNDS_UNIT};
use ledger::death_constraint::DeathProof;
use ledger_proof_statements::ptx::{PartialTxInputPrivate, PartialTxOutputPrivate};

const ZONE_SK: NullifierSecret = NullifierSecret::zero();

fn zone_state_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(goas_risc0_proofs::ZONE_STATE_ID)
}

fn zone_fund_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(
        goas_risc0_proofs::SPEND_ZONE_FUNDS_ID,
    )
}

fn zone_fund_in(
    value: u64,
    zone_metadata: ZoneMetadata,
    mut rng: impl rand_core::CryptoRngCore,
) -> cl::InputWitness {
    cl::InputWitness {
        note: cl::NoteWitness {
            value,
            unit: *common::ZONE_CL_FUNDS_UNIT,
            death_constraint: zone_metadata.funds_vk,
            state: zone_metadata.id(),
        },
        balance_blinding: BalanceWitness::unblinded(),
        nf_sk: ZONE_SK,
        nonce: NullifierNonce::random(&mut rng),
    }
}

#[test]
fn test_withdrawal() {
    let mut rng = rand::thread_rng();

    let zone_metadata = ZoneMetadata {
        zone_vk: zone_state_death_constraint(),
        funds_vk: zone_fund_death_constraint(),
        unit: cl::note::unit_point("Zone"),
    };

    let alice = 42;
    let alice_sk = NullifierSecret::random(&mut rng);

    let fund_in = zone_fund_in(35240, zone_metadata, &mut rng);

    let withdraw = common::Withdraw {
        from: alice,
        amount: 78,
        to: alice_sk.commit(),
        fund_nf: fund_in.nullifier(),
    };

    let zone_state = StateWitness {
        balances: BTreeMap::from_iter([(alice, 100)]),
        included_txs: vec![],
        output_events: vec![],
        zone_metadata,
    };

    let new_state = zone_state.clone().withdraw(withdraw);

    let zone_state_in = cl::InputWitness {
        note: cl::NoteWitness {
            value: 1,
            unit: zone_metadata.unit,
            death_constraint: zone_metadata.zone_vk,
            state: zone_state.commit().0,
        },
        balance_blinding: BalanceWitness::unblinded(),
        nf_sk: ZONE_SK,
        nonce: NullifierNonce::random(&mut rng),
    };

    let zone_state_out = cl::OutputWitness {
        note: cl::NoteWitness {
            state: new_state.commit().0,
            ..zone_state_in.note
        },
        balance_blinding: BalanceWitness::unblinded(),
        nf_pk: ZONE_SK.commit(),
        nonce: zone_state_in.nonce.evolve(&ZONE_SK),
    };

    let zone_fund_out = cl::OutputWitness {
        note: cl::NoteWitness {
            value: fund_in.note.value - withdraw.amount,
            ..fund_in.note
        },
        balance_blinding: BalanceWitness::unblinded(),
        nf_pk: ZONE_SK.commit(),
        nonce: fund_in.nonce.evolve(&ZONE_SK),
    };

    let alice_withdrawal = cl::OutputWitness {
        note: NoteWitness::stateless(
            withdraw.amount,
            *ZONE_CL_FUNDS_UNIT,
            DeathProof::nop_constraint(),
        ),
        balance_blinding: BalanceWitness::random(&mut rng),
        nf_pk: alice_sk.commit(),
        nonce: NullifierNonce::random(&mut rng),
    };

    let withdraw_ptx = cl::PartialTxWitness {
        inputs: vec![zone_state_in, fund_in],
        outputs: vec![zone_state_out, zone_fund_out, alice_withdrawal],
    };
    let withdraw_ptx_cm = withdraw_ptx.commit();

    let death_proofs = BTreeMap::from_iter([
        (
            zone_state_in.nullifier(),
            executor::prove_zone_stf(
                zone_state,
                vec![Input::Withdraw(withdraw)],
                PartialTxInputPrivate {
                    input: zone_state_in,
                    path: withdraw_ptx_cm.input_merkle_path(0), // merkle path to input state note (input #0)
                },
                PartialTxOutputPrivate {
                    output: zone_state_out,
                    path: withdraw_ptx_cm.output_merkle_path(0), // merkle path to output state note (output #0)
                },
            ),
        ),
        (
            fund_in.nullifier(),
            executor::prove_zone_fund_withdraw(
                PartialTxInputPrivate {
                    input: fund_in,
                    path: withdraw_ptx_cm.input_merkle_path(1), // merkle path to input fund note (input #1)
                },
                PartialTxOutputPrivate {
                    output: zone_state_out,
                    path: withdraw_ptx_cm.output_merkle_path(0),
                },
                PartialTxOutputPrivate {
                    output: zone_fund_out,
                    path: withdraw_ptx_cm.output_merkle_path(1),
                },
                PartialTxOutputPrivate {
                    output: alice_withdrawal,
                    path: withdraw_ptx_cm.output_merkle_path(2),
                },
                &new_state,
                withdraw,
            ),
        ),
    ]);

    let note_commitments = vec![zone_state_in.note_commitment(), fund_in.note_commitment()];

    let withdraw_proof =
        ledger::partial_tx::ProvedPartialTx::prove(&withdraw_ptx, death_proofs, &note_commitments)
            .expect("withdraw proof failed");

    assert!(withdraw_proof.verify());

    assert_eq!(withdraw_proof.outputs[0].output, zone_state_out.commit());
    assert_eq!(
        zone_state_out.note.state,
        StateWitness {
            balances: BTreeMap::from_iter([(alice, 22)]),
            included_txs: vec![Input::Withdraw(withdraw)],
            output_events: vec![Event::Spend(common::events::Spend {
                amount: 78,
                to: alice_sk.commit(),
                fund_nf: fund_in.nullifier()
            })],
            zone_metadata
        }
        .commit()
        .0
    )
}
