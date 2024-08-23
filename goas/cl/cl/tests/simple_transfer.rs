use cl::{note::derive_unit, BalanceWitness};
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
    let nmo = derive_unit("NMO");
    let mut rng = rand::thread_rng();

    let sender_nf_sk = cl::NullifierSecret::random(&mut rng);
    let sender_nf_pk = sender_nf_sk.commit();

    let recipient_nf_pk = cl::NullifierSecret::random(&mut rng).commit();

    // Assume the sender has received an unspent output from somewhere
    let utxo = receive_utxo(cl::NoteWitness::basic(10, nmo), sender_nf_pk, &mut rng);

    // and wants to send 8 NMO to some recipient and return 2 NMO to itself.
    let recipient_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(8, nmo), recipient_nf_pk, &mut rng);
    let change_output =
        cl::OutputWitness::random(cl::NoteWitness::basic(2, nmo), sender_nf_pk, &mut rng);

    let ptx_witness = cl::PartialTxWitness {
        inputs: vec![cl::InputWitness::from_output(utxo, sender_nf_sk)],
        outputs: vec![recipient_output, change_output],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    let bundle = cl::BundleWitness {
        partials: vec![ptx_witness],
    };

    assert!(bundle.balance().is_zero())
}
