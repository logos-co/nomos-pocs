use std::collections::BTreeMap;

use cl::{BalanceWitness, NoteWitness, NullifierSecret};
use common::{mmr::MMR, new_account, BoundTx, SignedBoundTx, StateWitness, Tx, ZONE_CL_FUNDS_UNIT};
use executor::ZoneNotes;
use ledger::constraint::ConstraintProof;

#[test]
fn test_withdrawal() {
    let mut rng = rand::thread_rng();

    let mut alice = new_account(&mut rng);
    let alice_vk = alice.verifying_key().to_bytes();
    let alice_cl_sk = NullifierSecret::random(&mut rng);
    let block_height = 0;

    let zone_start =
        ZoneNotes::new_with_balances("ZONE", BTreeMap::from_iter([(alice_vk, 100)]), &mut rng);

    let alice_intent = cl::InputWitness::from_output(
        cl::OutputWitness::new(
            NoteWitness::stateless(
                1,
                *ZONE_CL_FUNDS_UNIT,
                ConstraintProof::nop_constraint(),
                &mut rng,
            ), // TODO, intent should be in the constraint
            alice_cl_sk.commit(),
        ),
        alice_cl_sk,
    );

    let withdraw = common::Withdraw {
        from: alice_vk,
        amount: 78,
    };

    let zone_end = zone_start
        .clone()
        .run(block_height, Tx::Withdraw(withdraw))
        .0;

    let alice_withdrawal = cl::OutputWitness::new(
        NoteWitness::stateless(
            withdraw.amount,
            *ZONE_CL_FUNDS_UNIT,
            ConstraintProof::nop_constraint(),
            &mut rng,
        ),
        alice_cl_sk.commit(),
    );

    let withdraw_ptx = cl::PartialTxWitness {
        inputs: vec![
            zone_start.state_input_witness(),
            zone_start.fund_input_witness(),
            alice_intent,
        ],
        outputs: vec![zone_end.state_note, zone_end.fund_note, alice_withdrawal],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    let signed_withdraw = SignedBoundTx::sign(
        BoundTx {
            tx: Tx::Withdraw(withdraw),
            bind: alice_intent.note_commitment(),
        },
        &mut alice,
    );

    let constraint_proofs = BTreeMap::from_iter([
        (
            zone_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_start.state.clone(),
                block_height,
                vec![(signed_withdraw, withdraw_ptx.input_witness(2))],
                withdraw_ptx.input_witness(0), // input state note (input #0)
                withdraw_ptx.output_witness(0), // output state note (output #0)
                withdraw_ptx.output_witness(1), // output funds note (output #1)
            ),
        ),
        (
            zone_start.fund_input_witness().nullifier(),
            executor::prove_zone_fund_constraint(
                withdraw_ptx.input_witness(1),  // input fund note (input #1)
                withdraw_ptx.output_witness(0), // output state note (output #0)
                &zone_end.state,
                block_height,
            ),
        ),
        (
            alice_intent.nullifier(),
            ConstraintProof::prove_nop(
                alice_intent.nullifier(),
                withdraw_ptx.commit().root(),
                block_height,
            ),
        ),
    ]);

    let note_commitments = vec![
        zone_start.state_note.commit_note(),
        zone_start.fund_note.commit_note(),
        alice_intent.note_commitment(),
    ];

    let withdraw_proof = ledger::partial_tx::ProvedPartialTx::prove(
        &withdraw_ptx,
        constraint_proofs,
        &note_commitments,
    )
    .expect("withdraw proof failed");

    assert!(withdraw_proof.verify());

    assert_eq!(withdraw_proof.ptx.outputs[0], zone_end.state_note.commit());
    assert_eq!(
        zone_end.state_note.note.state,
        StateWitness {
            balances: BTreeMap::from_iter([(alice_vk, 22)]),
            ticket_auction: Default::default(),
            included_txs: {
                let mut mmr = MMR::new();
                mmr.push(&Tx::Withdraw(withdraw).to_bytes());
                mmr
            },
            zone_metadata: zone_start.state.zone_metadata,
        }
        .commit()
        .0
    )
}
