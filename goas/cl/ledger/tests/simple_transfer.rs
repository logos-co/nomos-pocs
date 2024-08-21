use std::collections::BTreeMap;

use cl::{note::unit_point, BalanceWitness};
use ledger::{bundle::ProvedBundle, death_constraint::DeathProof, partial_tx::ProvedPartialTx};
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

fn receive_utxo(
    note: cl::NoteWitness,
    nf_pk: cl::NullifierCommitment,
    rng: impl CryptoRngCore,
) -> cl::OutputWitness {
    cl::OutputWitness::random(note, nf_pk, rng)
}

#[test]
fn test_simple_transfer() {
    let nmo = unit_point("NMO");

    let mut rng = rand::thread_rng();

    // alice is sending 8 NMO to bob.

    let alice = User::random(&mut rng);
    let bob = User::random(&mut rng);

    // Alice has an unspent note worth 10 NMO
    let utxo = receive_utxo(
        cl::NoteWitness::stateless(10, nmo, DeathProof::nop_constraint()),
        alice.pk(),
        &mut rng,
    );
    let alices_input = cl::InputWitness::from_output(utxo, alice.sk());

    // Alice wants to send 8 NMO to bob
    let bobs_output = cl::OutputWitness::random(cl::NoteWitness::basic(8, nmo), bob.pk(), &mut rng);

    // .. and return the 2 NMO in change to herself.
    let change_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(2, nmo), alice.pk(), &mut rng);

    // Construct the ptx consuming Alices inputs and producing the two outputs.
    let ptx_witness = cl::PartialTxWitness {
        inputs: vec![alices_input],
        outputs: vec![bobs_output, change_output],
        balance_blinding: BalanceWitness::random(&mut rng),
    };

    // Prove the death constraints for alices input (she uses the no-op death constraint)
    let death_proofs = BTreeMap::from_iter(ptx_witness.inputs.iter().map(|i| {
        (
            i.nullifier(),
            DeathProof::prove_nop(i.nullifier(), ptx_witness.commit().root()),
        )
    }));

    // assume we only have one note commitment on chain for now ...
    let note_commitments = vec![utxo.commit_note()];
    let proved_ptx = ProvedPartialTx::prove(&ptx_witness, death_proofs, &note_commitments).unwrap();

    assert!(proved_ptx.verify()); // It's a valid ptx.

    let bundle = cl::Bundle {
        partials: vec![ptx_witness.commit()],
    };

    let bundle_witness = cl::BundleWitness {
        balance_blinding: ptx_witness.balance_blinding,
    };

    let proved_bundle = ProvedBundle::prove(&bundle, &bundle_witness).unwrap();
    assert!(proved_bundle.verify()); // The bundle is balanced.
}
