use ledger::{bundle::ProvedBundle, partial_tx::ProvedPartialTx};
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
    let mut rng = rand::thread_rng();

    // alice is sending 8 NMO to bob.

    let alice = User::random(&mut rng);
    let bob = User::random(&mut rng);

    // Alice has an unspent note worth 10 NMO
    let utxo = receive_utxo(cl::NoteWitness::basic(10, "NMO"), alice.pk(), &mut rng);
    let alices_input = cl::InputWitness::random(utxo, alice.sk(), &mut rng);

    // Alice wants to send 8 NMO to bob
    let bobs_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(8, "NMO"), bob.pk(), &mut rng);

    // .. and return the 2 NMO in change to herself.
    let change_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(2, "NMO"), alice.pk(), &mut rng);

    // Construct the ptx consuming Alices inputs and producing the two outputs.
    let ptx_witness = cl::PartialTxWitness {
        inputs: vec![alices_input],
        outputs: vec![bobs_output, change_output],
    };

    // assume we only have one note commitment on chain for now ...
    let note_commitments = vec![utxo.commit_note()];
    let proved_ptx = ProvedPartialTx::prove(&ptx_witness, &note_commitments);

    assert!(proved_ptx.verify()); // It's a valid ptx.

    let bundle = cl::Bundle {
        partials: vec![ptx_witness.commit()],
    };

    let proved_bundle = ProvedBundle::prove(&bundle, &ptx_witness.balance_blinding());
    assert!(proved_bundle.verify()); // The bundle is balanced.
}
