use cl::{
    cl::{
        balance::Unit, merkle, mmr::MMR, note::derive_unit, BalanceWitness, InputWitness,
        NoteWitness, NullifierCommitment, NullifierSecret, OutputWitness, PartialTxWitness,
    },
    zone_layer::{
        ledger::LedgerWitness,
        notes::{ZoneId, ZoneNote},
        tx::{UpdateBundle, ZoneUpdate},
    },
};
use ledger::{
    balance::ProvedBalance,
    constraint::ConstraintProof,
    ledger::{ProvedBundle, ProvedLedgerTransition},
    partial_tx::ProvedPartialTx,
    stf::StfProof,
    zone_update::ProvedUpdateBundle,
};
use ledger_proof_statements::{balance::BalancePrivate, stf::StfPublic};
use rand_core::CryptoRngCore;
use std::sync::OnceLock;

fn nmo() -> &'static Unit {
    static NMO: OnceLock<Unit> = OnceLock::new();
    NMO.get_or_init(|| derive_unit("NMO"))
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
    input_path: Vec<merkle::PathNode>,
    to: User,
    amount: u64,
    zone_a: ZoneId,
    zone_b: ZoneId,
    mut ledger_a: LedgerWitness,
    mut ledger_b: LedgerWitness,
) -> (ProvedLedgerTransition, ProvedLedgerTransition) {
    let mut rng = rand::thread_rng();
    assert!(amount <= input.note.value);
    let change = input.note.value - amount;
    let transfer = OutputWitness::new(
        NoteWitness::basic(amount, *nmo(), &mut rng),
        to.pk(),
        zone_b,
    );
    let change = OutputWitness::new(
        NoteWitness::basic(change, *nmo(), &mut rng),
        input.nf_sk.commit(),
        zone_a,
    );

    // Construct the ptx consuming the input and producing the two outputs.
    let ptx_witness = PartialTxWitness {
        inputs: vec![input],
        outputs: vec![transfer, change],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };
    let proved_ptx = ProvedPartialTx::prove(
        ptx_witness.clone(),
        vec![input_path],
        vec![ledger_a.commitments.roots[0].root],
    )
    .unwrap();

    let balance = ProvedBalance::prove(&BalancePrivate {
        balances: vec![ptx_witness.balance()],
    })
    .unwrap();

    let zone_tx = ProvedBundle {
        ptxs: vec![proved_ptx.clone()],
        balance,
    };

    // Prove the constraints for alices input (she uses the no-op constraint)
    let constraint_proof =
        ConstraintProof::prove_nop(input.nullifier(), proved_ptx.public.ptx.root());

    let ledger_a_transition = ProvedLedgerTransition::prove(
        ledger_a.clone(),
        zone_a,
        vec![zone_tx.clone()],
        vec![constraint_proof],
    )
    .unwrap();

    let ledger_b_transition =
        ProvedLedgerTransition::prove(ledger_b.clone(), zone_b, vec![zone_tx], vec![]).unwrap();

    ledger_a.commitments.push(&change.commit_note().0);
    ledger_a.nullifiers.push(input.nullifier());

    ledger_b.commitments.push(&transfer.commit_note().0);

    assert_eq!(ledger_a_transition.public.ledger, ledger_a.commit());
    assert_eq!(ledger_b_transition.public.ledger, ledger_b.commit());

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
        NoteWitness::stateless(10, *nmo(), ConstraintProof::nop_constraint(), &mut rng),
        alice.pk(),
        zone_a_id,
    );

    let alice_input = InputWitness::from_output(utxo, alice.sk());

    let mut mmr = MMR::new();
    let input_cm_path = mmr.push(&utxo.commit_note().0).path;

    let ledger_a = LedgerWitness {
        commitments: mmr,
        nullifiers: vec![],
    };

    let ledger_b = LedgerWitness {
        commitments: MMR::new(),
        nullifiers: vec![],
    };

    let zone_a_old = ZoneNote {
        id: zone_a_id,
        state: [0; 32],
        ledger: ledger_a.commit(),
        stf: [0; 32],
    };
    let zone_b_old = ZoneNote {
        id: zone_b_id,
        state: [0; 32],
        ledger: ledger_b.commit(),
        stf: [0; 32],
    };

    let (ledger_a_transition, ledger_b_transition) = cross_transfer_transition(
        alice_input,
        input_cm_path,
        bob,
        8,
        zone_a_id,
        zone_b_id,
        ledger_a,
        ledger_b,
    );

    let zone_a_new = ZoneNote {
        ledger: ledger_a_transition.public.ledger,
        ..zone_a_old
    };

    let zone_b_new = ZoneNote {
        ledger: ledger_b_transition.public.ledger,
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
