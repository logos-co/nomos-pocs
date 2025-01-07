use cl::{
    cl::{
        balance::Unit,
        mmr::{MMRProof, MMR},
        note::derive_unit,
        BalanceWitness, InputWitness, NoteWitness, NullifierCommitment, NullifierSecret,
        OutputWitness, PartialTxWitness,
    },
    zone_layer::{
        ledger::LedgerState,
        notes::{ZoneId, ZoneNote},
        tx::{UpdateBundle, ZoneUpdate},
    },
};
use ledger::{
    bundle::ProvedBundle, constraint::ConstraintProof, ledger::ProvedLedgerTransition,
    partial_tx::ProvedPartialTx, stf::StfProof, zone_update::ProvedUpdateBundle,
};
use ledger_proof_statements::{bundle::BundlePrivate, stf::StfPublic};
use rand_core::CryptoRngCore;
use std::sync::OnceLock;

fn nmo() -> Unit {
    static NMO: OnceLock<Unit> = OnceLock::new();
    *NMO.get_or_init(|| derive_unit("NMO"))
}

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

fn receive_utxo(note: NoteWitness, nf_pk: NullifierCommitment, zone_id: ZoneId) -> OutputWitness {
    OutputWitness::new(note, nf_pk, zone_id)
}

fn cross_transfer_transition(
    input: InputWitness,
    input_proof: (MMR, MMRProof),
    to: User,
    amount: u64,
    zone_a: ZoneId,
    zone_b: ZoneId,
    mut ledger_a: LedgerState,
    mut ledger_b: LedgerState,
) -> (ProvedLedgerTransition, ProvedLedgerTransition) {
    assert!(amount <= input.note.value);

    let mut rng = rand::thread_rng();

    let change = input.note.value - amount;
    let transfer = OutputWitness::new(NoteWitness::basic(amount, nmo(), &mut rng), to.pk(), zone_b);
    let change = OutputWitness::new(
        NoteWitness::basic(change, nmo(), &mut rng),
        input.nf_sk.commit(),
        zone_a,
    );

    // Construct the ptx consuming the input and producing the two outputs.
    let ptx_witness = PartialTxWitness {
        inputs: vec![input],
        outputs: vec![transfer, change],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    // Prove the constraints for alices input (she uses the no-op constraint)
    let constraint_proof =
        ConstraintProof::prove_nop(input.nullifier(), ptx_witness.commit().root());

    let proved_ptx = ProvedPartialTx::prove(
        ptx_witness.clone(),
        vec![input_proof],
        vec![constraint_proof.clone()],
    )
    .unwrap();

    let bundle = ProvedBundle::prove(
        &BundlePrivate {
            bundle: vec![proved_ptx.public()],
            balances: vec![ptx_witness.balance()],
        },
        vec![proved_ptx],
    );

    println!("proving ledger A transition");
    let ledger_a_transition =
        ProvedLedgerTransition::prove(ledger_a.clone(), zone_a, vec![bundle.clone()]);

    println!("proving ledger B transition");
    let ledger_b_transition = ProvedLedgerTransition::prove(ledger_b.clone(), zone_b, vec![bundle]);

    ledger_a.add_commitment(&change.commit_note());
    ledger_a.add_nullifiers(vec![input.nullifier()]);

    ledger_b.add_commitment(&transfer.commit_note());

    assert_eq!(
        ledger_a_transition.public().ledger,
        ledger_a.to_witness().commit()
    );
    assert_eq!(
        ledger_b_transition.public().ledger,
        ledger_b.to_witness().commit()
    );

    (ledger_a_transition, ledger_b_transition)
}

#[test]
fn zone_update_cross() {
    let mut rng = rand::thread_rng();

    let zone_a_id = [0; 32];
    let zone_b_id = [1; 32];

    // alice is sending 8 NMO to bob.

    let alice = User::random(&mut rng);
    let bob = User::random(&mut rng);

    // Alice has an unspent note worth 10 NMO
    let utxo = receive_utxo(
        NoteWitness::stateless(10, nmo(), ConstraintProof::nop_constraint(), &mut rng),
        alice.pk(),
        zone_a_id,
    );

    let alice_input = InputWitness::from_output(utxo, alice.sk());

    let mut ledger_a = LedgerState::default();
    let alice_cm_path = ledger_a.add_commitment(&utxo.commit_note());
    let alice_cm_proof = (ledger_a.commitments.clone(), alice_cm_path);

    let ledger_b = LedgerState::default();

    let zone_a_old = ZoneNote {
        id: zone_a_id,
        state: [0; 32],
        ledger: ledger_a.to_witness().commit(),
        stf: [0; 32],
    };
    let zone_b_old = ZoneNote {
        id: zone_b_id,
        state: [0; 32],
        ledger: ledger_b.to_witness().commit(),
        stf: [0; 32],
    };

    let (ledger_a_transition, ledger_b_transition) = cross_transfer_transition(
        alice_input,
        alice_cm_proof,
        bob,
        8,
        zone_a_id,
        zone_b_id,
        ledger_a,
        ledger_b,
    );

    let zone_a_new = ZoneNote {
        ledger: ledger_a_transition.public().ledger,
        ..zone_a_old
    };

    let zone_b_new = ZoneNote {
        ledger: ledger_b_transition.public().ledger,
        ..zone_b_old
    };

    let stf_proof_a = StfProof::prove_nop(StfPublic {
        old: zone_a_old,
        new: zone_a_new,
    });

    let stf_proof_b = StfProof::prove_nop(StfPublic {
        old: zone_b_old,
        new: zone_b_new,
    });

    let update_bundle = UpdateBundle {
        updates: vec![
            ZoneUpdate {
                old: zone_a_old,
                new: zone_a_new,
            },
            ZoneUpdate {
                old: zone_b_old,
                new: zone_b_new,
            },
        ],
    };

    let proved_bundle = ProvedUpdateBundle {
        bundle: update_bundle,
        ledger_proofs: vec![ledger_a_transition, ledger_b_transition],
        stf_proofs: vec![stf_proof_a, stf_proof_b],
    };

    assert!(proved_bundle.verify());
}
