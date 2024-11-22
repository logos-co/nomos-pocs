use cl::{
    cl::{
        note::derive_unit, BalanceWitness, BundleWitness, InputWitness, NoteWitness,
        NullifierCommitment, NullifierSecret, OutputWitness, PartialTxWitness,
    },
    zone_layer::ledger::LedgerWitness,
};
use ledger::{
    bundle::ProvedBundle,
    constraint::ConstraintProof,
    ledger::{ProvedLedgerTransition, ProvedZoneTx},
    partial_tx::ProvedPartialTx,
};
use rand_core::CryptoRngCore;

struct User(NullifierSecret);

impl User {
    fn random(mut rng: impl CryptoRngCore) -> Self {
        Self(NullifierSecret::random(&mut rng))
    }

    fn pk(&self) -> NullifierCommitment {
        self.0.commit()
    }

    fn sk(&self) -> NullifierSecret {
        self.0
    }
}

fn receive_utxo(note: NoteWitness, nf_pk: NullifierCommitment) -> OutputWitness {
    OutputWitness::new(note, nf_pk)
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
        NoteWitness::stateless(10, nmo, ConstraintProof::nop_constraint(), &mut rng),
        alice.pk(),
    );

    // Alice wants to send 8 NMO to bob
    let bobs_output = OutputWitness::new(NoteWitness::basic(8, nmo, &mut rng), bob.pk());

    let alice_change = OutputWitness::new(NoteWitness::basic(2, nmo, &mut rng), alice.pk());

    let alice_input = InputWitness::from_output(utxo, alice.sk());

    let zone_a = [0; 32];
    let zone_b = [1; 32];
    let ledger_a = LedgerWitness {
        commitments: vec![utxo.commit_note(&zone_a)],
        nullifiers: vec![],
    };

    let ledger_b = LedgerWitness {
        commitments: vec![],
        nullifiers: vec![],
    };

    let expected_ledger_a = LedgerWitness {
        commitments: vec![utxo.commit_note(&zone_a), alice_change.commit_note(&zone_a)],
        nullifiers: vec![alice_input.nullifier(&zone_a)],
    };

    let expected_ledger_b = LedgerWitness {
        commitments: vec![bobs_output.commit_note(&zone_b)],
        nullifiers: vec![],
    };

    // Construct the ptx consuming Alices inputs and producing the two outputs.
    let alice_ptx_witness = PartialTxWitness {
        inputs: vec![alice_input],
        outputs: vec![bobs_output, alice_change],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };
    let proved_ptx = ProvedPartialTx::prove(
        alice_ptx_witness.clone(),
        vec![ledger_a
            .cm_path(&alice_input.note_commitment(&zone_a))
            .unwrap()],
        ledger_a.cm_root(),
        vec![zone_a],
        vec![zone_b, zone_a],
    )
    .unwrap();

    let bundle = ProvedBundle::prove(&BundleWitness {
        partials: vec![alice_ptx_witness],
    })
    .unwrap();

    assert_eq!(proved_ptx.cm_root, ledger_a.cm_root());

    let zone_tx = ProvedZoneTx {
        ptxs: vec![proved_ptx.clone()],
        bundle,
    };

    // Prove the constraints for alices input (she uses the no-op constraint)
    let constraint_proof =
        ConstraintProof::prove_nop(alice_input.nullifier(&zone_a), proved_ptx.ptx.root());

    let ledger_a_transition = ProvedLedgerTransition::prove(
        ledger_a,
        zone_a,
        vec![zone_tx.clone()],
        vec![constraint_proof],
    )
    .unwrap();

    let ledger_b_transition =
        ProvedLedgerTransition::prove(ledger_b, zone_b, vec![zone_tx], vec![]).unwrap();

    assert_eq!(
        ledger_a_transition.public.ledger,
        expected_ledger_a.commit()
    );
    assert_eq!(
        ledger_b_transition.public.ledger,
        expected_ledger_b.commit()
    );
}
