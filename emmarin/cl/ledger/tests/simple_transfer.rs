use cl::{
    crust::{
        balance::{UnitWitness, NOP_COVENANT},
        BundleWitness, InputWitness, NoteCommitment, Nullifier, NullifierCommitment,
        NullifierSecret, OutputWitness, TxWitness,
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
use rand::{Rng, RngCore};
use rand_core::CryptoRngCore;

const ZONE_A: ZoneId = [0u8; 32];
const ZONE_B: ZoneId = [1u8; 32];

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
    ledger_in: &mut LedgerState,
    ledger_out: &mut LedgerState,
) -> (ProvedLedgerTransition, ProvedLedgerTransition) {
    assert!(amount <= input.value);
    let mut rng = rand::thread_rng();

    let (transfer, change) =
        OutputWitness::spend_with_change(input, amount, to.pk(), to_zone, &mut rng);

    let tx_witness = TxWitness::default()
        .add_input(input, input_proof)
        .add_output(transfer, vec![])
        .add_output(change, vec![]);

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
        ProvedLedgerTransition::prove(ledger_in, input.zone_id, vec![bundle.clone()]);

    println!("proving ledger B transition");
    let ledger_out_transition = ProvedLedgerTransition::prove(ledger_out, to_zone, vec![bundle]);

    (ledger_in_transition, ledger_out_transition)
}

struct ZoneWitness {
    ledger: LedgerState,
}

impl ZoneWitness {
    fn new() -> Self {
        let ledger = LedgerState::default();

        Self { ledger }
    }

    fn state(&self) -> ZoneState {
        ZoneState {
            stf: StfProof::nop_stf(),
            zone_data: [0; 32],
            ledger: self.ledger.to_witness().commit(),
        }
    }

    fn fill_nfs(&mut self, amount: usize, mut rng: impl RngCore) {
        self.ledger.add_nullifiers(
            std::iter::repeat_with(|| Nullifier(rng.gen()))
                .take(amount)
                .collect(),
        );
    }

    fn add_commitment(&mut self, cm: &NoteCommitment) -> (MMR, MMRProof) {
        self.ledger.add_commitment(cm)
    }
}

#[test]
fn zone_update_cross() {
    let mut rng = rand::thread_rng();

    // Alice is sending 8 NMO to bob.
    let alice = User::random(&mut rng);
    let bob = User::random(&mut rng);

    // Alice has an unspent note worth 10 NMO
    let utxo = OutputWitness::new(10, nmo().unit(), alice.pk(), ZONE_A, &mut rng);

    let alice_input = InputWitness::from_output(utxo, alice.sk(), nmo());

    let mut zone_a = ZoneWitness::new();
    zone_a.fill_nfs(2_usize.pow(10), &mut rng);
    let alice_cm_proof = zone_a.add_commitment(&utxo.note_commitment());

    let mut zone_b = ZoneWitness::new();

    let (zone_a_old, zone_b_old) = (zone_a.state(), zone_b.state());

    let (ledger_proof_a, ledger_proof_b) = cross_transfer_transition(
        alice_input,
        alice_cm_proof,
        bob,
        8,
        ZONE_B,
        &mut zone_a.ledger,
        &mut zone_b.ledger,
    );

    let (zone_a_new, zone_b_new) = (zone_a.state(), zone_b.state());

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
        ledger_proofs: vec![ledger_proof_a, ledger_proof_b],
        stf_proofs: vec![stf_proof_a, stf_proof_b],
    };

    assert!(proved_batch.verify());
}
