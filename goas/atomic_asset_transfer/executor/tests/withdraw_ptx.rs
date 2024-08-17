use std::collections::BTreeMap;

use cl::{NoteWitness, NullifierSecret};
use common::{new_account, BoundTx, SignedBoundTx, StateWitness, Tx, ZONE_CL_FUNDS_UNIT};
use executor::ZoneNotes;
use ledger::death_constraint::DeathProof;

#[test]
fn test_withdrawal() {
    let mut rng = rand::thread_rng();

    let mut alice = new_account(&mut rng);
    let alice_vk = alice.verifying_key().to_bytes();
    let alice_cl_sk = NullifierSecret::random(&mut rng);

    let zone_start =
        ZoneNotes::new_with_balances("ZONE", BTreeMap::from_iter([(alice_vk, 100)]), &mut rng);

    let alice_intent = cl::InputWitness::random(
        cl::OutputWitness::random(
            NoteWitness::stateless(1, *ZONE_CL_FUNDS_UNIT, DeathProof::nop_constraint()), // TODO, intent should be in the death constraint
            alice_cl_sk.commit(),
            &mut rng,
        ),
        alice_cl_sk,
        &mut rng,
    );

    let withdraw = common::Withdraw {
        from: alice_vk,
        amount: 78,
    };

    let zone_end = zone_start.clone().run([Tx::Withdraw(withdraw)]);

    let alice_withdrawal = cl::OutputWitness::random(
        NoteWitness::stateless(
            withdraw.amount,
            *ZONE_CL_FUNDS_UNIT,
            DeathProof::nop_constraint(),
        ),
        alice_cl_sk.commit(),
        &mut rng,
    );

    let withdraw_ptx = cl::PartialTxWitness {
        inputs: vec![
            zone_start.state_input_witness(),
            zone_start.fund_input_witness(),
            alice_intent,
        ],
        outputs: vec![zone_end.state_note, zone_end.fund_note, alice_withdrawal],
    };

    let signed_withdraw = SignedBoundTx::sign(
        BoundTx {
            tx: Tx::Withdraw(withdraw),
            bind: alice_intent.note_commitment(),
        },
        &mut alice,
    );

    let death_proofs = BTreeMap::from_iter([
        (
            zone_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_start.state.clone(),
                vec![(signed_withdraw, withdraw_ptx.input_witness(2))],
                withdraw_ptx.input_witness(0), // input state note (input #0)
                withdraw_ptx.output_witness(0), // output state note (output #0)
                withdraw_ptx.output_witness(1), // output funds note (output #1)
            ),
        ),
        (
            zone_start.fund_input_witness().nullifier(),
            executor::prove_zone_fund_withdraw(
                withdraw_ptx.input_witness(1),  // input fund note (input #1)
                withdraw_ptx.output_witness(0), // output state note (output #0)
                &zone_end.state,
            ),
        ),
        (
            alice_intent.nullifier(),
            DeathProof::prove_nop(alice_intent.nullifier(), withdraw_ptx.commit().root()),
        ),
    ]);

    let note_commitments = vec![
        zone_start.state_note.commit_note(),
        zone_start.fund_note.commit_note(),
        alice_intent.note_commitment(),
    ];

    let withdraw_proof =
        ledger::partial_tx::ProvedPartialTx::prove(&withdraw_ptx, death_proofs, &note_commitments)
            .expect("withdraw proof failed");

    assert!(withdraw_proof.verify());

    assert_eq!(
        withdraw_proof.outputs[0].output,
        zone_end.state_note.commit()
    );
    assert_eq!(
        zone_end.state_note.note.state,
        StateWitness {
            balances: BTreeMap::from_iter([(alice_vk, 22)]),
            included_txs: vec![Tx::Withdraw(withdraw)],
            zone_metadata: zone_start.state.zone_metadata,
        }
        .commit()
        .0
    )
}
