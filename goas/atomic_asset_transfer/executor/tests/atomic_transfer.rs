use std::collections::BTreeMap;

use cl::{BalanceWitness, BundleWitness, NoteWitness, NullifierNonce};
use common::{new_account, BoundTx, Deposit, SignedBoundTx, Tx, Withdraw};
use executor::ZoneNotes;
use goas_proof_statements::user_note::{UserAtomicTransfer, UserIntent};

#[test]
fn test_atomic_transfer() {
    let mut rng = rand::thread_rng();

    let mut alice = new_account(&mut rng);
    let alice_vk = alice.verifying_key().to_bytes();

    let zone_a_start =
        ZoneNotes::new_with_balances("ZONE_A", BTreeMap::from_iter([(alice_vk, 100)]), &mut rng);

    let zone_b_start = ZoneNotes::new_with_balances("ZONE_B", BTreeMap::from_iter([]), &mut rng);

    let alice_intent = UserIntent {
        zone_a_meta: zone_a_start.state.zone_metadata,
        zone_b_meta: zone_b_start.state.zone_metadata,
        withdraw: Withdraw {
            from: alice_vk,
            amount: 75,
        },
        deposit: Deposit {
            to: alice_vk,
            amount: 75,
        },
    };

    let alice_intent_out = cl::OutputWitness::public(
        NoteWitness {
            value: 1,
            unit: cl::note::unit_point("INTENT"),
            death_constraint: executor::user_atomic_transfer_death_constraint(),
            state: alice_intent.commit(),
        },
        NullifierNonce::random(&mut rng),
    );

    let user_ptx = cl::PartialTxWitness {
        inputs: vec![],
        outputs: vec![alice_intent_out],
        balance_blinding: BalanceWitness::random(&mut rng),
    };

    let zone_a_end = zone_a_start
        .clone()
        .run([Tx::Withdraw(alice_intent.withdraw)]);

    let zone_b_end = zone_b_start
        .clone()
        .run([Tx::Deposit(alice_intent.deposit)]);

    let alice_intent_in = cl::InputWitness::public(alice_intent_out);
    let atomic_transfer_ptx = cl::PartialTxWitness {
        inputs: vec![
            alice_intent_in,
            zone_a_start.state_input_witness(),
            zone_a_start.fund_input_witness(),
            zone_b_start.state_input_witness(),
            zone_b_start.fund_input_witness(),
        ],
        outputs: vec![
            zone_a_end.state_note,
            zone_a_end.fund_note,
            zone_b_end.state_note,
            zone_b_end.fund_note,
        ],
        balance_blinding: BalanceWitness::random(&mut rng),
    };

    let signed_withdraw = SignedBoundTx::sign(
        BoundTx {
            tx: Tx::Withdraw(alice_intent.withdraw),
            bind: alice_intent_in.note_commitment(),
        },
        &mut alice,
    );
    let signed_deposit = SignedBoundTx::sign(
        BoundTx {
            tx: Tx::Deposit(alice_intent.deposit),
            bind: alice_intent_in.note_commitment(),
        },
        &mut alice,
    );

    let death_proofs = BTreeMap::from_iter([
        (
            alice_intent_in.nullifier(),
            executor::prove_user_atomic_transfer(UserAtomicTransfer {
                user_note: atomic_transfer_ptx.input_witness(0),
                user_intent: alice_intent,
                zone_a: atomic_transfer_ptx.output_witness(0),
                zone_b: atomic_transfer_ptx.output_witness(2),
                zone_a_roots: zone_a_end.state.state_roots(),
                zone_b_roots: zone_b_end.state.state_roots(),
                withdraw_tx: zone_a_end.state.included_tx_witness(0),
                deposit_tx: zone_b_end.state.included_tx_witness(0),
            }),
        ),
        (
            zone_a_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_a_start.state.clone(),
                vec![(signed_withdraw, atomic_transfer_ptx.input_witness(0))], // withdraw bound to input intent note
                atomic_transfer_ptx.input_witness(1),                          // input state note
                atomic_transfer_ptx.output_witness(0),                         // output state note
                atomic_transfer_ptx.output_witness(1),                         // output funds note
            ),
        ),
        (
            zone_a_start.fund_input_witness().nullifier(),
            executor::prove_zone_fund_constraint(
                atomic_transfer_ptx.input_witness(2),  // input fund note
                atomic_transfer_ptx.output_witness(0), // output state note
                &zone_a_end.state,
            ),
        ),
        (
            zone_b_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_b_start.state.clone(),
                vec![(signed_deposit, atomic_transfer_ptx.input_witness(0))], // deposit bound to input intent note
                atomic_transfer_ptx.input_witness(3),                         // input state note
                atomic_transfer_ptx.output_witness(2),                        // output state note
                atomic_transfer_ptx.output_witness(3),                        // output funds note
            ),
        ),
        (
            zone_b_start.fund_input_witness().nullifier(),
            executor::prove_zone_fund_constraint(
                atomic_transfer_ptx.input_witness(4), // input fund note (input #1)
                atomic_transfer_ptx.output_witness(2), // output state note (output #0)
                &zone_b_end.state,
            ),
        ),
    ]);

    let user_ptx_proof =
        ledger::partial_tx::ProvedPartialTx::prove(&user_ptx, BTreeMap::new(), &[])
            .expect("user ptx failed to prove");
    assert!(user_ptx_proof.verify());

    let note_commitments = vec![
        alice_intent_out.commit_note(),
        zone_a_start.state_note.commit_note(),
        zone_a_start.fund_note.commit_note(),
        zone_b_start.state_note.commit_note(),
        zone_b_start.fund_note.commit_note(),
    ];

    let atomic_transfer_proof = ledger::partial_tx::ProvedPartialTx::prove(
        &atomic_transfer_ptx,
        death_proofs,
        &note_commitments,
    )
    .expect("atomic transfer proof failed");

    assert!(atomic_transfer_proof.verify());

    let bundle = cl::Bundle {
        partials: vec![user_ptx.commit(), atomic_transfer_ptx.commit()],
    };

    let bundle_witness = BundleWitness {
        balance_blinding: cl::BalanceWitness(
            user_ptx.balance_blinding.0 + atomic_transfer_ptx.balance_blinding.0,
        ),
    };

    let bundle_proof =
        ledger::bundle::ProvedBundle::prove(&bundle, &bundle_witness).expect("bundle proof failed");

    assert!(bundle_proof.verify());
}
