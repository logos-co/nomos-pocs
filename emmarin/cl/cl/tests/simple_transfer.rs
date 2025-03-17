use cl::crust::{InputWitness, Nonce, NullifierSecret, OutputWitness, TxWitness, UnitWitness};

fn nmo_unit() -> UnitWitness {
    UnitWitness {
        spending_covenant: [0; 32],
        minting_covenant: [0; 32],
        burning_covenant: [0; 32],
        arg: [0; 32],
    }
}

#[test]
fn test_simple_transfer() {
    let nmo = nmo_unit();
    let mut rng = rand::thread_rng();
    let zone_id = [0; 32];

    let sender_nf_sk = NullifierSecret::random(&mut rng);
    let sender_nf_pk = sender_nf_sk.commit();

    let recipient_nf_pk = NullifierSecret::random(&mut rng).commit();

    // Assume the sender has received an unspent output from somewhere
    let utxo = OutputWitness {
        state: [0; 32],
        value: 10,
        unit: nmo.unit(),
        nonce: Nonce::random(&mut rng),
        zone_id,
        nf_pk: sender_nf_pk,
    };

    // and wants to send 8 NMO to some recipient and return 2 NMO to itself.
    let recipient_output = OutputWitness {
        state: [0; 32],
        value: 8,
        unit: nmo.unit(),
        nonce: Nonce::random(&mut rng),
        zone_id,
        nf_pk: recipient_nf_pk,
    };
    let change_output = OutputWitness {
        state: [0; 32],
        value: 2,
        unit: nmo.unit(),
        nonce: Nonce::random(&mut rng),
        zone_id,
        nf_pk: sender_nf_pk,
    };

    let tx_witness = TxWitness {
        inputs: vec![InputWitness::from_output(utxo, sender_nf_sk, nmo)],
        outputs: vec![(recipient_output, Vec::new()), (change_output, Vec::new())],
        data: vec![],
        mints: vec![],
        burns: vec![],
        frontier_paths: vec![],
    };

    assert!(tx_witness
        .balance(&tx_witness.mint_amounts(), &tx_witness.burn_amounts())
        .is_zero())
}
