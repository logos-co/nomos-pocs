//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template additive_note_slahsing(){

    //note
    signal input state;
    signal input value;
    signal input nonce;
    signal input secret_key;

    signal input pending_balance;   // pending balance between 0 and 2**64 for rewards and p-2**64 and p for slashing

    signal output previous_commitment;
    signal output new_commitment;


    component pk = derive_public_key();
    pk.secret_key <== secret_key;

    component previous_cm = commitment();
    component new_cm = commitment();
    component nmo = NMO();
    component staking = STAKING();

    // Derive the commitment of the note before changing the value
    previous_cm.state <== state;
    previous_cm.value <== value;
    previous_cm.unit <== nmo.out;
    previous_cm.nonce <== nonce;
    previous_cm.zoneID <== staking.out;
    previous_cm.public_key <== pk.out;
    previous_commitment <== previous_cm.out;


    // Derive the new commitment after secret key modification
    // The ownership is the same because both secret keys are known
    new_cm.state <== state;
    new_cm.value <== value + pending_balance;
    new_cm.unit <== nmo.out;
    new_cm.nonce <== nonce;
    new_cm.zoneID <== staking.out;
    new_cm.public_key <== pk.out;
    new_commitment <== new_cm.out;
}

component main {public [pending_balance]}= additive_note_slahsing();