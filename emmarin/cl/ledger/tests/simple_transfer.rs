use cl::{note::derive_unit, BalanceWitness};
use ledger::{
    constraint::ConstraintProof,
    ledger::{ProvedLedgerTransition, ProvedZoneTx},
    pact::ProvedPact,
};
use rand_core::CryptoRngCore;

struct User(cl::NullifierSecret);

impl User {
    fn random(mut rng: impl CryptoRngCore) -> Self {
        Self(cl::NullifierSecret::random(&mut rng))
    }

    fn pk(&self) -> cl::NullifierCommitment {
        self.0.commit()
    }

    fn sk(&self) -> cl::NullifierSecret {
        self.0
    }
}

fn receive_utxo(note: cl::NoteWitness, nf_pk: cl::NullifierCommitment) -> cl::OutputWitness {
    cl::OutputWitness::new(note, nf_pk)
}

#[test]
fn ledger_transition() {
    let nmo = derive_unit("NMO");

    let mut rng = rand::thread_rng();

    // alice is sending 8 NMO to bob.

    let alice = User::random(&mut rng);
    let bob = User::random(&mut rng);

    // Alice has an unspent note worth 10 NMO
    let utxo = receive_utxo(
        cl::NoteWitness::stateless(10, nmo, ConstraintProof::nop_constraint(), &mut rng),
        alice.pk(),
    );

    // Alice wants to send 8 NMO to bob
    let bobs_output = cl::OutputWitness::new(cl::NoteWitness::basic(8, nmo, &mut rng), bob.pk());

    let alice_change = cl::OutputWitness::new(cl::NoteWitness::basic(2, nmo, &mut rng), alice.pk());

    let alice_input = cl::InputWitness::from_output(utxo, alice.sk());

    let zone_a = [0; 32];
    let zone_b = [1; 32];
    let ledger_a = cl::zones::LedgerWitness {
        commitments: vec![utxo.commit_note(&zone_a)],
        nullifiers: vec![],
    };

    let ledger_b = cl::zones::LedgerWitness {
        commitments: vec![],
        nullifiers: vec![],
    };

    let expected_ledger_a = cl::zones::LedgerWitness {
        commitments: vec![utxo.commit_note(&zone_a), alice_change.commit_note(&zone_a)],
        nullifiers: vec![alice_input.nullifier(&zone_a)],
    };

    let expected_ledger_b = cl::zones::LedgerWitness {
        commitments: vec![bobs_output.commit_note(&zone_b)],
        nullifiers: vec![],
    };

    // Construct the ptx consuming Alices inputs and producing the two outputs.
    let alice_pact_witness = cl::zones::PactWitness {
        tx: cl::PartialTxWitness {
            inputs: vec![alice_input],
            outputs: vec![bobs_output, alice_change],

            balance_blinding: BalanceWitness::random_blinding(&mut rng),
        },
        from: zone_a,
        to: vec![zone_b, zone_a],
    };

    let proved_pact = ProvedPact::prove(
        alice_pact_witness,
        vec![ledger_a
            .cm_path(&alice_input.note_commitment(&zone_a))
            .unwrap()],
        ledger_a.cm_root(),
    )
    .unwrap();

    assert_eq!(proved_pact.cm_root, ledger_a.cm_root());

    // Prove the constraints for alices input (she uses the no-op constraint)
    let constraint_proof =
        ConstraintProof::prove_nop(alice_input.nullifier(&zone_a), proved_pact.pact.tx.root());

    let ledger_a_transition = ProvedLedgerTransition::prove(
        ledger_a,
        zone_a,
        vec![ProvedZoneTx::Pact(proved_pact.clone())],
        vec![constraint_proof],
    )
    .unwrap();

    let ledger_b_transition = ProvedLedgerTransition::prove(
        ledger_b,
        zone_b,
        vec![ProvedZoneTx::Pact(proved_pact)],
        vec![],
    )
    .unwrap();

    assert_eq!(
        ledger_a_transition.public.ledger,
        expected_ledger_a.commit()
    );
    assert_eq!(
        ledger_b_transition.public.ledger,
        expected_ledger_b.commit()
    );
}
