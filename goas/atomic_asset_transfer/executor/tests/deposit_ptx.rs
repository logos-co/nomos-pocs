use std::collections::BTreeMap;

use cl::{NoteWitness, NullifierSecret};
use common::{new_account, BoundTx, SignedBoundTx, StateWitness, Tx, ZONE_CL_FUNDS_UNIT};
use executor::ZoneNotes;
use ledger::death_constraint::DeathProof;

#[test]
fn test_deposit() {
    let mut rng = rand::thread_rng();

    let mut alice = new_account(&mut rng);
    let alice_vk = alice.verifying_key().to_bytes();
    let alice_cl_sk = NullifierSecret::random(&mut rng);

    let zone_start = ZoneNotes::new_with_balances("ZONE", BTreeMap::new(), &mut rng);

    let deposit = common::Deposit {
        to: alice_vk,
        amount: 78,
    };

    let zone_end = zone_start.clone().run([Tx::Deposit(deposit)]);

    let alice_deposit = cl::InputWitness::random(
        cl::OutputWitness::random(
            NoteWitness::stateless(
                78,
                *ZONE_CL_FUNDS_UNIT,
                DeathProof::nop_constraint(), // alice should demand a tx inclusion proof for the deposit
            ),
            alice_cl_sk.commit(),
            &mut rng,
        ),
        alice_cl_sk,
        &mut rng,
    );

    let deposit_ptx = cl::PartialTxWitness {
        inputs: vec![zone_start.state_input_witness(), alice_deposit],
        outputs: vec![zone_end.state_note, zone_end.fund_note],
    };

    let signed_deposit = SignedBoundTx::sign(
        BoundTx {
            tx: Tx::Deposit(deposit),
            bind: alice_deposit.note_commitment(),
        },
        &mut alice,
    );

    let death_proofs = BTreeMap::from_iter([
        (
            zone_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_start.state.clone(),
                vec![(signed_deposit, deposit_ptx.input_witness(1))], // bind it to the deposit note)],
                deposit_ptx.input_witness(0),                         // input state note (input #0)
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
        zone_start.state_note.commit_note(),
        alice_deposit.note_commitment(),
    ];

    let deposit_proof =
        ledger::partial_tx::ProvedPartialTx::prove(&deposit_ptx, death_proofs, &note_commitments)
            .expect("deposit proof failed");

    assert!(deposit_proof.verify());

    assert_eq!(
        deposit_proof.outputs[0].output,
        zone_end.state_note.commit()
    );
    assert_eq!(
        zone_end.state_note.note.state,
        StateWitness {
            balances: BTreeMap::from_iter([(alice_vk, 78)]),
            included_txs: vec![Tx::Deposit(deposit)],
            zone_metadata: zone_start.state.zone_metadata,
        }
        .commit()
        .0
    );
    assert_eq!(
        deposit_ptx.commit().balance(),
        cl::Balance::zero(deposit_ptx.balance_blinding())
    );
}
