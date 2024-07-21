use ledger::{bundle::ProvedBundle, partial_tx::ProvedPartialTx};
use rand_core::CryptoRngCore;

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

    let sender_nf_sk = cl::NullifierSecret::random(&mut rng);
    let sender_nf_pk = sender_nf_sk.commit();

    let recipient_nf_pk = cl::NullifierSecret::random(&mut rng).commit();

    // Assume the sender has received an unspent output from somewhere
    let utxo = receive_utxo(cl::NoteWitness::basic(10, "NMO"), sender_nf_pk, &mut rng);

    let note_commitments = vec![utxo.commit_note()];

    let input = cl::InputWitness::random(utxo, sender_nf_sk, &mut rng);

    // and wants to send 8 NMO to some recipient and return 2 NMO to itself.
    let recipient_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(8, "NMO"), recipient_nf_pk, &mut rng);
    let change_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(2, "NMO"), sender_nf_pk, &mut rng);

    let ptx_witness = cl::PartialTxWitness {
        inputs: vec![input],
        outputs: vec![recipient_output, change_output],
    };

    let proved_ptx = ProvedPartialTx::prove(&ptx_witness, &note_commitments);

    assert!(proved_ptx.verify());

    let bundle = cl::Bundle {
        partials: vec![ptx_witness.commit()],
    };

    let proved_bundle = ProvedBundle::prove(&bundle, &ptx_witness.balance_blinding());
    assert!(proved_bundle.verify());
}
