use std::collections::BTreeMap;

use cl::{BalanceWitness, NoteWitness, NullifierSecret};
use common::{mmr::MMR, new_account, BoundTx, SignedBoundTx, StateWitness, Tx, ZONE_CL_FUNDS_UNIT};
use executor::ZoneNotes;
use ledger::constraint::ConstraintProof;

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

    let zone_end = zone_start.clone().run(Tx::Deposit(deposit)).0;

    let alice_deposit = cl::InputWitness::from_output(
        cl::OutputWitness::new(
            NoteWitness::stateless(
                78,
                *ZONE_CL_FUNDS_UNIT,
                ConstraintProof::nop_constraint(), // alice should demand a tx inclusion proof for the deposit
                &mut rng,
            ),
            alice_cl_sk.commit(),
        ),
        alice_cl_sk,
    );

    let deposit_ptx = cl::PartialTxWitness {
        inputs: vec![zone_start.state_input_witness(), alice_deposit],
        outputs: vec![zone_end.state_note, zone_end.fund_note],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    let signed_deposit = SignedBoundTx::sign(
        BoundTx {
            tx: Tx::Deposit(deposit),
            bind: alice_deposit.note_commitment(),
        },
        &mut alice,
    );

    let constraint_proofs = BTreeMap::from_iter([
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
        &deposit_ptx,
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
