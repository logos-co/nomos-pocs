use std::collections::BTreeMap;

use cl::{note::derive_unit, BalanceWitness};
use ledger::{bundle::ProvedBundle, constraint::ConstraintProof, partial_tx::ProvedPartialTx};
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
fn test_simple_transfer() {
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
    let alices_input = cl::InputWitness::from_output(utxo, alice.sk());

    // Alice wants to send 8 NMO to bob
    let bobs_output = cl::OutputWitness::new(cl::NoteWitness::basic(8, nmo, &mut rng), bob.pk());

    // .. and return the 2 NMO in change to herself.
    let change_output =
        cl::OutputWitness::new(cl::NoteWitness::basic(2, nmo, &mut rng), alice.pk());

    // Construct the ptx consuming Alices inputs and producing the two outputs.
    let ptx_witness = cl::PartialTxWitness {
        inputs: vec![alices_input],
        outputs: vec![bobs_output, change_output],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    // Prove the constraints for alices input (she uses the no-op constraint)
    let constraint_proofs = BTreeMap::from_iter(ptx_witness.inputs.iter().map(|i| {
        (
            i.nullifier(),
            ConstraintProof::prove_nop(i.nullifier(), ptx_witness.commit().root()),
        )
    }));

    // assume we only have one note commitment on chain for now ...
    let note_commitments = vec![utxo.commit_note()];
    let proved_ptx =
        ProvedPartialTx::prove(&ptx_witness, constraint_proofs, &note_commitments).unwrap();

    assert!(proved_ptx.verify()); // It's a valid ptx.

    let bundle_witness = cl::BundleWitness {
        partials: vec![ptx_witness],
    };

    let proved_bundle = ProvedBundle::prove(&bundle_witness).unwrap();
    assert!(proved_bundle.verify()); // The bundle is balanced.
}
