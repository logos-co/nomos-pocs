use cl::{
    crust::{
        balance::{UnitWitness, NOP_COVENANT},
        BundleWitness, InputWitness, Nonce, Nullifier, NullifierCommitment, NullifierSecret,
        OutputWitness, TxWitness,
    },
    ds::mmr::{MMRProof, MMR},
    mantle::{
        ledger::LedgerState,
        update::{BatchUpdate, Update},
        ZoneId, ZoneState,
    },
};
use ledger::{
    bundle::ProvedBundle, ledger::ProvedLedgerTransition, stf::StfProof, tx::ProvedTx,
    update::ProvedBatchUpdate,
};
use ledger_proof_statements::stf::StfPublic;
use rand::Rng;
use rand_core::CryptoRngCore;

fn nmo() -> UnitWitness {
    UnitWitness {
        spending_covenant: NOP_COVENANT,
        minting_covenant: NOP_COVENANT,
        burning_covenant: NOP_COVENANT,
    }
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

fn cross_transfer_transition(
    input: InputWitness,
    input_proof: (MMR, MMRProof),
    to: User,
    amount: u64,
    to_zone: ZoneId,
    mut ledger_in: LedgerState,
    mut ledger_out: LedgerState,
) -> (ProvedLedgerTransition, ProvedLedgerTransition) {
    assert!(amount <= input.value);
    println!("nfs in zone_a: {}", ledger_in.nullifiers.len());
    println!("nfs in zone_b: {}", ledger_out.nullifiers.len());

    let mut rng = rand::thread_rng();

    let change = input.value - amount;
    let transfer = OutputWitness {
        state: Default::default(),
        value: amount,
        unit: nmo().unit(),
        nonce: Nonce::random(&mut rng),
        zone_id: to_zone,
        nf_pk: to.pk(),
    };
    let change = OutputWitness {
        state: Default::default(),
        value: change,
        unit: nmo().unit(),
        nonce: Nonce::random(&mut rng),
        zone_id: input.zone_id,
        nf_pk: input.nf_sk.commit(), // return change to sender
    };

    // Construct the tx consuming the input and producing the two outputs.
    let tx_witness = TxWitness {
        inputs: vec![input],
        outputs: vec![(transfer, vec![]), (change, vec![])],
        data: Default::default(),
        mints: vec![],
        burns: vec![],
        frontier_paths: vec![input_proof],
    };

    let proved_tx = ProvedTx::prove(
        tx_witness.clone(),
        vec![],
        vec![], // we can skip covenant proofs since NMO uses no-op spend covenants
    )
    .unwrap();

    let bundle = ProvedBundle::prove(
        &BundleWitness {
            txs: vec![proved_tx.public()],
        },
        vec![proved_tx],
    );

    println!("proving ledger A transition");
    let ledger_in_transition =
        ProvedLedgerTransition::prove(ledger_in.clone(), input.zone_id, vec![bundle.clone()]);

    println!("proving ledger B transition");
    let ledger_out_transition =
        ProvedLedgerTransition::prove(ledger_out.clone(), to_zone, vec![bundle]);

    ledger_in.add_commitment(&change.note_commitment());
    ledger_in.add_nullifiers(vec![input.nullifier()]);

    ledger_out.add_commitment(&transfer.note_commitment());

    assert_eq!(
        ledger_in_transition.public().ledger,
        ledger_in.to_witness().commit()
    );
    assert_eq!(
        ledger_out_transition.public().ledger,
        ledger_out.to_witness().commit()
    );

    (ledger_in_transition, ledger_out_transition)
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
    let utxo = OutputWitness {
        state: Default::default(),
        value: 10,
        unit: nmo().unit(),
        nonce: Nonce::random(&mut rng),
        zone_id: zone_a_id,
        nf_pk: alice.pk(),
    };

    let alice_input = InputWitness::from_output(utxo, alice.sk(), nmo());

    let mut ledger_a = LedgerState::default();
    ledger_a.add_nullifiers(
        std::iter::repeat_with(|| Nullifier(rng.gen()))
            .take(2_usize.pow(20))
            .collect(),
    );
    let alice_cm_path = ledger_a.add_commitment(&utxo.note_commitment());
    let alice_cm_proof = (ledger_a.commitments.clone(), alice_cm_path);

    let ledger_b = LedgerState::default();

    let zone_a_old = ZoneState {
        stf: StfProof::nop_stf(),
        zone_data: [0; 32],
        ledger: ledger_a.to_witness().commit(),
    };

    let zone_b_old = ZoneState {
        stf: StfProof::nop_stf(),
        zone_data: [0; 32],
        ledger: ledger_b.to_witness().commit(),
    };

    let (ledger_a_transition, ledger_b_transition) = cross_transfer_transition(
        alice_input,
        alice_cm_proof,
        bob,
        8,
        zone_b_id,
        ledger_a,
        ledger_b,
    );

    let zone_a_new = ZoneState {
        ledger: ledger_a_transition.public().ledger,
        ..zone_a_old
    };

    let zone_b_new = ZoneState {
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

    let batch = BatchUpdate {
        updates: vec![
            Update {
                old: zone_a_old,
                new: zone_a_new,
            },
            Update {
                old: zone_b_old,
                new: zone_b_new,
            },
        ],
    };

    let proved_batch = ProvedBatchUpdate {
        batch,
        ledger_proofs: vec![ledger_a_transition, ledger_b_transition],
        stf_proofs: vec![stf_proof_a, stf_proof_b],
    };

    assert!(proved_batch.verify());
}
