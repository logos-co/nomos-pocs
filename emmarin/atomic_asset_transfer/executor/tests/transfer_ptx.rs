use std::collections::BTreeMap;

use cl::{BalanceWitness, NoteWitness, NullifierSecret};
use common::{mmr::MMR, new_account, BoundTx, SignedBoundTx, StateWitness, Tx, ZONE_CL_FUNDS_UNIT};
use executor::ZoneNotes;
use ledger::constraint::ConstraintProof;

#[test]
fn test_transfer() {
    let mut rng = rand::thread_rng();

    let mut alice = new_account(&mut rng);
    let alice_vk = alice.verifying_key().to_bytes();
    let alice_cl_sk = NullifierSecret::random(&mut rng);

    let zone_id = [0; 32];

    let alice_note = cl::InputWitness::from_output(
        cl::OutputWitness::new(
            NoteWitness::stateless(
                1,
                *ZONE_CL_FUNDS_UNIT,
                ConstraintProof::nop_constraint(),
                &mut rng,
            ),
            alice_cl_sk.commit(),
        ),
        alice_cl_sk,
    );

    let foundation_sk = NullifierSecret::random(&mut rng);
    let foundation_pk = foundation_sk.commit();

    let mut ledger = LedgerWitness::new();
    ledger.commitments.push(alice_note.note_commitment(zone_id));

    let zone_start = ZoneNote::new_with_pks("ZONE", vec![charity_pk], ledger, &mut rng);

    let output = cl::OutputWitness::new(
        NoteWitness::stateless(
            1,
            *ZONE_CL_FUNDS_UNIT,
            ConstraintProof::nop_constraint(),
            &mut rng,
        ),
        foundation_sk.commit(),
    );

    let zone_transfer_auth = Tx::Transfer(Transfer {
        nf: alice_note.nullifier(),
        cm: output.commit(zone_id),
    });

    let transfer = cl::PartialTxWitness {
        inputs: vec![zone_start.zone_input_witness(), alice_note],
        outputs: vec![zone_end.zone_note, output],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    let constraint_proofs = BTreeMap::from_iter([
        (
            zone_start.zone_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_start.state.clone(),
                vec![zone_transfer_auth],
                transfer.input_witness(0),  // input state note (input #0)
                transfer.output_witness(0), // output state note (output #0)
                transfer.output_witness(1), // output funds note (output #1)
            ),
        ),
        (
            alice_deposit.nullifier(),
            ledger::ConstraintProof::prove_nop(
                alice_deposit.nullifier(),
                deposit_ptx.commit().root(),
            ),
        ),
    ]);

    let note_commitments = vec![
        zone_start.state_note.commit_note(),
        alice_deposit.note_commitment(),
    ];

    let deposit_proof = ledger::partial_tx::ProvedPartialTx::prove(
        &transfer_ptx,
        constraint_proofs,
        &note_commitments,
    )
    .expect("deposit proof failed");

    assert!(deposit_proof.verify());

    assert_eq!(deposit_proof.ptx.outputs[0], zone_end.state_note.commit());
    assert_eq!(
        zone_end.state_note.note.state,
        StateWitness {
            balances: BTreeMap::from_iter([(alice_vk, 78)]),
            included_txs: {
                let mut mmr = MMR::new();
                mmr.push(&Tx::Deposit(deposit).to_bytes());
                mmr
            },
            zone_metadata: zone_start.state.zone_metadata,
        }
        .commit()
        .0
    );
    assert!(deposit_ptx.balance().is_zero());
}
