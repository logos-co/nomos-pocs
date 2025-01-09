use cl::{
    cl::{
        balance::Unit,
        mmr::{MMRProof, UpdateableMMRProof, MMR},
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

#[derive(Clone, Copy, Debug)]
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
    inputs: Vec<InputWitness>,
    input_proofs: Vec<(MMR, MMRProof)>,
    to: &[User],
    amounts: &[u64],
    zone_a: ZoneId,
    zone_b: ZoneId,
    ledger_a: LedgerState,
    ledger_b: LedgerState,
) -> (ProvedLedgerTransition, ProvedLedgerTransition) {
    // assert!(amount <= input.note.value);

    let mut rng = rand::thread_rng();

    let mut outputs = Vec::new();

    let mut expected_ledger_a = ledger_a.clone();
    let mut expected_ledger_b = ledger_b.clone();

    for ((input, amount), to) in inputs.iter().zip(amounts).zip(to) {
        let change = input.note.value - amount;
        let transfer = OutputWitness::new(
            NoteWitness::basic(*amount, nmo(), &mut rng),
            to.pk(),
            zone_b,
        );
        let change = OutputWitness::new(
            NoteWitness::basic(change, nmo(), &mut rng),
            input.nf_sk.commit(),
            zone_a,
        );

        expected_ledger_a.add_commitment(&change.commit_note());
        expected_ledger_b.add_commitment(&transfer.commit_note());
        outputs.extend([transfer, change]);
    }

    let mut nullifiers = inputs
        .iter()
        .map(|input| input.nullifier())
        .collect::<Vec<_>>();
    nullifiers.sort();
    expected_ledger_a.add_nullifiers(nullifiers);

    // Construct the ptx consuming the input and producing the two outputs.
    let ptx_witness = PartialTxWitness {
        inputs: inputs.clone(),
        outputs,
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    // Prove the constraints for alices input (she uses the no-op constraint)
    let constraint_proofs = inputs
        .iter()
        .map(|input| ConstraintProof::prove_nop(input.nullifier(), ptx_witness.commit().root()))
        .collect::<Vec<_>>();

    let proved_ptx =
        ProvedPartialTx::prove(ptx_witness.clone(), input_proofs, constraint_proofs.clone())
            .unwrap();

    let bundle = ProvedBundle::prove(
        &BundlePrivate {
            bundle: vec![proved_ptx.public()],
            balances: vec![ptx_witness.balance()],
        },
        vec![proved_ptx],
    );

    println!("proving ledger B transition");
    let ledger_b_transition =
        ProvedLedgerTransition::prove(ledger_b.clone(), zone_b, vec![bundle.clone()]);

    println!("proving ledger A transition");
    let ledger_a_transition =
        ProvedLedgerTransition::prove(ledger_a.clone(), zone_a, vec![bundle.clone()]);

    assert_eq!(
        ledger_a_transition.public().ledger,
        expected_ledger_a.to_witness().commit()
    );
    assert_eq!(
        ledger_b_transition.public().ledger,
        expected_ledger_b.to_witness().commit()
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

    // Alice has 8 unspent notes worth 10 NMO
    let utxos = (0..4)
        .map(|_| {
            receive_utxo(
                NoteWitness::stateless(10, nmo(), ConstraintProof::nop_constraint(), &mut rng),
                alice.pk(),
                zone_a_id,
            )
        })
        .collect::<Vec<_>>();

    let inputs = utxos
        .iter()
        .map(|utxo| InputWitness::from_output(utxo.clone(), alice.sk()))
        .collect::<Vec<_>>();

    let mut ledger_a = LedgerState::default();
    let mut nullifiers = Vec::new();
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    for _ in 0..1 << 20 {
        let mut nf = [0; 32];
        rng.fill_bytes(&mut nf);
        nullifiers.push(cl::cl::Nullifier(nf));
    }

    ledger_a.add_nullifiers(nullifiers);

    let mut cm_proofs: Vec<UpdateableMMRProof> = Vec::new();

    for utxo in &utxos {
        for proof in &mut cm_proofs {
            proof.update(&utxo.commit_note().0);
        }
        let proof = ledger_a.add_commitment(&utxo.commit_note());
        cm_proofs.push(proof);
    }

    for proof in &cm_proofs {
        println!("proof: {:?}", proof);
    }

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

    let to = std::iter::repeat(bob)
        .take(inputs.len())
        .collect::<Vec<_>>();
    let amounts = std::iter::repeat(8).take(inputs.len()).collect::<Vec<_>>();

    let (ledger_a_transition, ledger_b_transition) = cross_transfer_transition(
        inputs,
        cm_proofs.into_iter().map(|up| (up.mmr, up.proof)).collect(),
        &to,
        &amounts,
        zone_a_id,
        zone_b_id,
        ledger_a,
        ledger_b,
    );

    // let zone_a_new = ZoneNote {
    //     ledger: ledger_a_transition.public().ledger,
    //     ..zone_a_old
    // };

    // let zone_b_new = ZoneNote {
    //     ledger: ledger_b_transition.public().ledger,
    //     ..zone_b_old
    // };

    // let stf_proof_a = StfProof::prove_nop(StfPublic {
    //     old: zone_a_old,
    //     new: zone_a_new,
    // });

    // let stf_proof_b = StfProof::prove_nop(StfPublic {
    //     old: zone_b_old,
    //     new: zone_b_new,
    // });

    // let update_bundle = UpdateBundle {
    //     updates: vec![
    //         ZoneUpdate {
    //             old: zone_a_old,
    //             new: zone_a_new,
    //         },
    //         ZoneUpdate {
    //             old: zone_b_old,
    //             new: zone_b_new,
    //         },
    //     ],
    // };

    // let proved_bundle = ProvedUpdateBundle {
    //     bundle: update_bundle,
    //     ledger_proofs: vec![ledger_a_transition, ledger_b_transition],
    //     stf_proofs: vec![stf_proof_a, stf_proof_b],
    // };

    // assert!(proved_bundle.verify());
}
