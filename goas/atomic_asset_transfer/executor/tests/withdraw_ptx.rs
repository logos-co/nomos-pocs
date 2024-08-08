use std::collections::{BTreeMap, VecDeque};

use cl::{NoteWitness, NullifierNonce, NullifierSecret};
use common::{Input, StateWitness, ZoneMetadata, ZONE_CL_FUNDS_UNIT};
use ledger::death_constraint::DeathProof;
use rand_core::CryptoRngCore;

fn zone_state_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(goas_risc0_proofs::ZONE_STATE_ID)
}

fn zone_fund_death_constraint() -> [u8; 32] {
    ledger::death_constraint::risc0_id_to_cl_death_constraint(
        goas_risc0_proofs::SPEND_ZONE_FUNDS_ID,
    )
}

fn zone_fund_utxo(
    value: u64,
    zone_meta: ZoneMetadata,
    mut rng: impl CryptoRngCore,
) -> cl::OutputWitness {
    cl::OutputWitness::public(
        cl::NoteWitness {
            value,
            unit: *common::ZONE_CL_FUNDS_UNIT,
            death_constraint: zone_meta.funds_vk,
            state: zone_meta.id(),
        },
        NullifierNonce::random(&mut rng),
    )
}

fn zone_state_utxo(zone: &StateWitness, mut rng: impl CryptoRngCore) -> cl::OutputWitness {
    cl::OutputWitness::public(
        cl::NoteWitness {
            value: 1,
            unit: zone.zone_metadata.unit,
            death_constraint: zone.zone_metadata.zone_vk,
            state: zone.commit().0,
        },
        NullifierNonce::random(&mut rng),
    )
}

#[test]
fn test_withdrawal() {
    let mut rng = rand::thread_rng();

    let alice = 42;
    let alice_sk = NullifierSecret::random(&mut rng);

    let init_state = StateWitness {
        balances: BTreeMap::from_iter([(alice, 100)]),
        included_txs: vec![],
        zone_metadata: ZoneMetadata {
            zone_vk: zone_state_death_constraint(),
            funds_vk: zone_fund_death_constraint(),
            unit: cl::note::unit_point("ZONE_STATE"),
        },
        nonce: [0; 32],
    };

    let zone_fund_in =
        cl::InputWitness::public(zone_fund_utxo(100, init_state.zone_metadata, &mut rng));
    let zone_state_in = cl::InputWitness::public(zone_state_utxo(&init_state, &mut rng));

    let withdraw = common::Withdraw {
        from: alice,
        amount: 78,
        to: alice_sk.commit(),
    };

    let end_state = init_state.clone().withdraw(withdraw).evolve_nonce();

    let zone_state_out = cl::OutputWitness::public(
        cl::NoteWitness {
            state: end_state.commit().0,
            ..zone_state_in.note
        },
        zone_state_in.evolved_nonce(),
    );
    let zone_fund_out = cl::OutputWitness::public(
        cl::NoteWitness {
            value: zone_fund_in.note.value - withdraw.amount,
            ..zone_fund_in.note
        },
        NullifierNonce::from_bytes(end_state.nonce),
    );

    let alice_withdrawal = cl::OutputWitness::random(
        NoteWitness::stateless(
            withdraw.amount,
            *ZONE_CL_FUNDS_UNIT,
            DeathProof::nop_constraint(),
        ),
        alice_sk.commit(),
        &mut rng,
    );

    let withdraw_ptx = cl::PartialTxWitness {
        inputs: vec![zone_state_in, zone_fund_in],
        outputs: vec![zone_state_out, zone_fund_out, alice_withdrawal],
    };

    let death_proofs = BTreeMap::from_iter([
        (
            zone_state_in.nullifier(),
            executor::prove_zone_stf(
                init_state.clone(),
                vec![Input::Withdraw(withdraw)],
                withdraw_ptx.input_witness(0), // input state note (input #0)
                withdraw_ptx.output_witness(0), // output state note (output #0)
                withdraw_ptx.output_witness(1), // output funds note (output #1)
                VecDeque::from_iter([withdraw_ptx.output_witness(2)]), // alice withdrawal
                VecDeque::new(),               // no deposits
            ),
        ),
        (
            zone_fund_in.nullifier(),
            executor::prove_zone_fund_withdraw(
                withdraw_ptx.input_witness(1),  // input fund note (input #1)
                withdraw_ptx.output_witness(0), // output state note (output #0)
                &end_state,
            ),
        ),
    ]);

    let note_commitments = vec![
        zone_state_in.note_commitment(),
        zone_fund_in.note_commitment(),
    ];

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
            zone_metadata: init_state.zone_metadata,
            nonce: init_state.evolve_nonce().nonce,
        }
        .commit()
        .0
    )
}
