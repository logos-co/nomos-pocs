use std::collections::BTreeMap;

use cl::{BundleWitness, NoteWitness, NullifierNonce};
use common::{BoundTx, Deposit, StateWitness, Tx, Withdraw, ZoneMetadata};
use goas_proof_statements::user_note::UserIntent;
use rand_core::CryptoRngCore;

fn zone_fund_utxo(
    value: u64,
    zone_meta: ZoneMetadata,
    mut rng: impl CryptoRngCore,
) -> cl::OutputWitness {
    cl::OutputWitness::public(
        cl::NoteWitness {
            value,
            unit: *common::ZONE_CL_FUNDS_UNIT,
            death_constraint: zone_meta.funds_vk,
            state: zone_meta.id(),
        },
        NullifierNonce::random(&mut rng),
    )
}

fn zone_state_utxo(zone: &StateWitness, mut rng: impl CryptoRngCore) -> cl::OutputWitness {
    cl::OutputWitness::public(
        cl::NoteWitness {
            value: 1,
            unit: zone.zone_metadata.unit,
            death_constraint: zone.zone_metadata.zone_vk,
            state: zone.commit().0,
        },
        NullifierNonce::random(&mut rng),
    )
}

#[derive(Debug, Clone)]
struct ZoneNotes {
    state: StateWitness,
    state_note: cl::OutputWitness,
    fund_note: cl::OutputWitness,
}

impl ZoneNotes {
    fn new_with_balances(
        zone_name: &str,
        balances: BTreeMap<u32, u64>,
        mut rng: impl CryptoRngCore,
    ) -> Self {
        let state = StateWitness {
            balances,
            included_txs: vec![],
            zone_metadata: executor::zone_metadata(zone_name),
            nonce: [0; 32],
        };
        let state_note = zone_state_utxo(&state, &mut rng);
        let fund_note = zone_fund_utxo(state.total_balance(), state.zone_metadata, &mut rng);
        Self {
            state,
            state_note,
            fund_note,
        }
    }

    fn state_input_witness(&self) -> cl::InputWitness {
        cl::InputWitness::public(self.state_note)
    }

    fn fund_input_witness(&self) -> cl::InputWitness {
        cl::InputWitness::public(self.fund_note)
    }

    fn run(mut self, txs: Vec<Tx>) -> Self {
        for tx in txs {
            self.state = self.state.apply(tx);
        }
        self.state = self.state.evolve_nonce();

        let state_in = self.state_input_witness();
        self.state_note = cl::OutputWitness::public(
            cl::NoteWitness {
                state: self.state.commit().0,
                ..state_in.note
            },
            state_in.evolved_nonce(),
        );

        let fund_in = self.fund_input_witness();
        self.fund_note = cl::OutputWitness::public(
            cl::NoteWitness {
                value: self.state.total_balance(),
                ..fund_in.note
            },
            NullifierNonce::from_bytes(self.state.nonce),
        );
        self
    }
}

#[test]
fn test_atomic_transfer() {
    let mut rng = rand::thread_rng();

    let alice = 42;

    let zone_a_start =
        ZoneNotes::new_with_balances("ZONE_A", BTreeMap::from_iter([(alice, 100)]), &mut rng);

    let zone_b_start = ZoneNotes::new_with_balances("ZONE_B", BTreeMap::from_iter([]), &mut rng);

    let alice_intent = UserIntent {
        zone_a_meta: zone_a_start.state.zone_metadata,
        zone_b_meta: zone_b_start.state.zone_metadata,
        withdraw: Withdraw {
            from: alice,
            amount: 75,
        },
        deposit: Deposit {
            to: alice,
            amount: 75,
        },
    };

    let alice_intent_out = cl::OutputWitness::public(
        NoteWitness {
            value: 1,
            unit: cl::note::unit_point("INTENT"),
            death_constraint: executor::user_atomic_transfer_death_constraint(),
            state: alice_intent.commit(),
        },
        NullifierNonce::random(&mut rng),
    );

    let user_ptx = cl::PartialTxWitness {
        inputs: vec![],
        outputs: vec![alice_intent_out],
    };

    let zone_a_end = zone_a_start
        .clone()
        .run(vec![Tx::Withdraw(alice_intent.withdraw)]);

    let zone_b_end = zone_b_start
        .clone()
        .run(vec![Tx::Deposit(alice_intent.deposit)]);

    let alice_intent_in = cl::InputWitness::public(alice_intent_out);
    let atomic_transfer_ptx = cl::PartialTxWitness {
        inputs: vec![
            alice_intent_in,
            zone_a_start.state_input_witness(),
            zone_a_start.fund_input_witness(),
            zone_b_start.state_input_witness(),
            zone_b_start.fund_input_witness(),
        ],
        outputs: vec![
            zone_a_end.state_note,
            zone_a_end.fund_note,
            zone_b_end.state_note,
            zone_b_end.fund_note,
        ],
    };

    let death_proofs = BTreeMap::from_iter([
        (
            alice_intent_in.nullifier(),
            executor::prove_user_atomic_transfer(
                atomic_transfer_ptx.input_witness(0),
                alice_intent,
                atomic_transfer_ptx.output_witness(0),
                atomic_transfer_ptx.output_witness(2),
                zone_a_end.state.state_roots(),
                zone_b_end.state.state_roots(),
                zone_a_end.state.included_tx_witness(0),
                zone_b_end.state.included_tx_witness(0),
            ),
        ),
        (
            zone_a_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_a_start.state.clone(),
                vec![BoundTx {
                    tx: Tx::Withdraw(alice_intent.withdraw),
                    bind: atomic_transfer_ptx.input_witness(0), // input intent note
                }],
                atomic_transfer_ptx.input_witness(1), // input state note
                atomic_transfer_ptx.output_witness(0), // output state note
                atomic_transfer_ptx.output_witness(1), // output funds note
            ),
        ),
        (
            zone_a_start.fund_input_witness().nullifier(),
            executor::prove_zone_fund_withdraw(
                atomic_transfer_ptx.input_witness(2),  // input fund note
                atomic_transfer_ptx.output_witness(0), // output state note
                &zone_a_end.state,
            ),
        ),
        (
            zone_b_start.state_input_witness().nullifier(),
            executor::prove_zone_stf(
                zone_b_start.state.clone(),
                vec![BoundTx {
                    tx: Tx::Deposit(alice_intent.deposit),
                    bind: atomic_transfer_ptx.input_witness(0), // input intent note
                }],
                atomic_transfer_ptx.input_witness(3), // input state note
                atomic_transfer_ptx.output_witness(2), // output state note
                atomic_transfer_ptx.output_witness(3), // output funds note
            ),
        ),
        (
            zone_b_start.fund_input_witness().nullifier(),
            executor::prove_zone_fund_withdraw(
                atomic_transfer_ptx.input_witness(4), // input fund note (input #1)
                atomic_transfer_ptx.output_witness(2), // output state note (output #0)
                &zone_b_end.state,
            ),
        ),
    ]);

    let user_ptx_proof =
        ledger::partial_tx::ProvedPartialTx::prove(&user_ptx, BTreeMap::new(), &[])
            .expect("user ptx failed to prove");
    assert!(user_ptx_proof.verify());

    let note_commitments = vec![
        alice_intent_out.commit_note(),
        zone_a_start.state_note.commit_note(),
        zone_a_start.fund_note.commit_note(),
        zone_b_start.state_note.commit_note(),
        zone_b_start.fund_note.commit_note(),
    ];

    let atomic_transfer_proof = ledger::partial_tx::ProvedPartialTx::prove(
        &atomic_transfer_ptx,
        death_proofs,
        &note_commitments,
    )
    .expect("atomic transfer proof failed");

    assert!(atomic_transfer_proof.verify());

    let bundle = cl::Bundle {
        partials: vec![user_ptx.commit(), atomic_transfer_ptx.commit()],
    };

    let bundle_witness = BundleWitness {
        balance_blinding: cl::BalanceWitness(
            user_ptx.balance_blinding().0 + atomic_transfer_ptx.balance_blinding().0,
        ),
    };

    let bundle_proof =
        ledger::bundle::ProvedBundle::prove(&bundle, &bundle_witness).expect("bundle proof failed");

    assert!(bundle_proof.verify());
}
