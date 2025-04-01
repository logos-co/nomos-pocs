//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template note_refreshment(){

    //note
    signal input state;
    signal input value;
    signal input nonce;
    signal input previous_sk;
    signal input new_sk;
    signal input zoneID;

    signal output previous_commitment;
    signal output new_commitment;

    // Derive the two public keys proving ownership of the keys
    component previous_pk = derive_public_key();
    component new_pk = derive_public_key();
    previous_pk.secret_key <== previous_sk;
    new_pk.secret_key <== new_sk;

    component previous_cm = commitment();
    component new_cm = commitment();
    component nmo = NMO();

    // Derive the commitment of the note before changing the key
    previous_cm.state <== state;
    previous_cm.value <== value;
    previous_cm.unit <== nmo.out;
    previous_cm.nonce <== nonce;
    previous_cm.zoneID <== zoneID;
    previous_cm.public_key <== previous_pk.out;
    previous_commitment <== previous_cm.out;


    // Derive the new commitment after secret key modification
    // The ownership is the same because both secret keys are known
    new_cm.state <== state;
    new_cm.value <== value;
    new_cm.unit <== nmo.out;
    new_cm.nonce <== nonce;
    new_cm.zoneID <== zoneID;
    new_cm.public_key <== new_pk.out;
    new_commitment <== new_cm.out;
}

component main {public [zoneID]}= note_refreshment();