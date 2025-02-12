use cl::{
    cl::{
        note::derive_unit, BalanceWitness, InputWitness, NoteWitness, NullifierCommitment,
        NullifierSecret, OutputWitness, TxWitness,
    },
    zone_layer::notes::ZoneId,
};

fn receive_utxo(note: NoteWitness, nf_pk: NullifierCommitment, zone_id: ZoneId) -> OutputWitness {
    OutputWitness::new(note, nf_pk, zone_id)
}

#[test]
fn test_simple_transfer() {
    let nmo = derive_unit("NMO");
    let mut rng = rand::thread_rng();
    let zone_id = [0; 32];

    let sender_nf_sk = NullifierSecret::random(&mut rng);
    let sender_nf_pk = sender_nf_sk.commit();

    let recipient_nf_pk = NullifierSecret::random(&mut rng).commit();

    // Assume the sender has received an unspent output from somewhere
    let utxo = receive_utxo(NoteWitness::basic(10, nmo, &mut rng), sender_nf_pk, zone_id);

    // and wants to send 8 NMO to some recipient and return 2 NMO to itself.
    let recipient_output = OutputWitness::new(
        NoteWitness::basic(8, nmo, &mut rng),
        recipient_nf_pk,
        zone_id,
    );
    let change_output =
        OutputWitness::new(NoteWitness::basic(2, nmo, &mut rng), sender_nf_pk, zone_id);

    let ptx_witness = TxWitness {
        inputs: vec![InputWitness::from_output(utxo, sender_nf_sk)],
        outputs: vec![recipient_output, change_output],
        balance_blinding: BalanceWitness::random_blinding(&mut rng),
    };

    assert!(ptx_witness.balance().is_zero())
}
