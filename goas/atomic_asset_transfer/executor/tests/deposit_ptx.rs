use std::collections::BTreeMap;

use cl::{NoteWitness, NullifierNonce, NullifierSecret};
use common::{BoundTx, StateWitness, Tx, ZoneMetadata, ZONE_CL_FUNDS_UNIT};
use ledger::death_constraint::DeathProof;
use rand_core::CryptoRngCore;

fn zone_fund_note(value: u64, zone_meta: ZoneMetadata) -> cl::NoteWitness {
    cl::NoteWitness {
        value,
        unit: *common::ZONE_CL_FUNDS_UNIT,
        death_constraint: zone_meta.funds_vk,
        state: zone_meta.id(),
    }
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
fn test_deposit() {
    let mut rng = rand::thread_rng();

    let alice = 42;
    let alice_sk = NullifierSecret::random(&mut rng);

    let init_state = StateWitness {
        balances: BTreeMap::new(),
        included_txs: vec![],
        zone_metadata: executor::zone_metadata("ZONE"),
        nonce: [0; 32],
    };

    let zone_state_in = cl::InputWitness::public(zone_state_utxo(&init_state, &mut rng));

    let deposit = common::Deposit {
        to: alice,
        amount: 78,
    };

    let end_state = init_state.clone().deposit(deposit).evolve_nonce();

    let zone_state_out = cl::OutputWitness::public(
        cl::NoteWitness {
            state: end_state.commit().0,
            ..zone_state_in.note
        },
        zone_state_in.evolved_nonce(),
    );
    let zone_fund_out = cl::OutputWitness::public(
        zone_fund_note(78, init_state.zone_metadata),
        NullifierNonce::from_bytes(end_state.nonce),
    );

    let mut alice_state = [0u8; 32];
    alice_state[..4].copy_from_slice(&alice.to_le_bytes());

    let alice_deposit = cl::InputWitness::random(
        cl::OutputWitness::random(
            NoteWitness::new(
                78,
                *ZONE_CL_FUNDS_UNIT,
                DeathProof::nop_constraint(), // alice should demand a tx inclusion proof for the deposit
                alice_state,
            ),
            alice_sk.commit(),
            &mut rng,
        ),
        alice_sk,
        &mut rng,
    );

    let deposit_ptx = cl::PartialTxWitness {
        inputs: vec![zone_state_in, alice_deposit],
        outputs: vec![zone_state_out, zone_fund_out],
    };

    let death_proofs = BTreeMap::from_iter([
        (
            zone_state_in.nullifier(),
            executor::prove_zone_stf(
                init_state.clone(),
                vec![BoundTx {
                    tx: Tx::Deposit(deposit),
                    bind: deposit_ptx.input_witness(1), // bind it to the deposit note
                }],
                deposit_ptx.input_witness(0), // input state note (input #0)
                deposit_ptx.output_witness(0), // output state note (output #0)
                deposit_ptx.output_witness(1), // output funds note (output #1)
            ),
        ),
        (
            alice_deposit.nullifier(),
            ledger::DeathProof::prove_nop(alice_deposit.nullifier(), deposit_ptx.commit().root()),
        ),
    ]);

    let note_commitments = vec![
        zone_state_in.note_commitment(),
        alice_deposit.note_commitment(),
    ];

    let deposit_proof =
        ledger::partial_tx::ProvedPartialTx::prove(&deposit_ptx, death_proofs, &note_commitments)
            .expect("deposit proof failed");

    assert!(deposit_proof.verify());

    assert_eq!(deposit_proof.outputs[0].output, zone_state_out.commit());
    assert_eq!(
        zone_state_out.note.state,
        StateWitness {
            balances: BTreeMap::from_iter([(alice, 78)]),
            included_txs: vec![Tx::Deposit(deposit)],
            zone_metadata: init_state.zone_metadata,
            nonce: init_state.evolve_nonce().nonce,
        }
        .commit()
        .0
    );
    assert_eq!(
        deposit_ptx.commit().balance(),
        cl::Balance::zero(deposit_ptx.balance_blinding())
    );
}
