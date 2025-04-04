//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "merkle.circom";
include "../misc/constants.circom";

// The unit of the note is supposed to be NMO
template commitment(){
    signal input state;
    signal input value;
    signal input unit;
    signal input nonce;
    signal input zoneID;
    signal input public_key;
    signal output out;

    component hash = Poseidon2_hash(7);
    component dst = NOMOS_NOTE_CM();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== state;
    hash.inp[2] <== value;
    hash.inp[3] <== unit;
    hash.inp[4] <== nonce;
    hash.inp[5] <== public_key;
    hash.inp[6] <== zoneID;

    out <== hash.out;
}

template nullifier(){
    signal input commitment;
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(3);
    component dst = NOMOS_NOTE_NF();
    hash.inp[0] <==  dst.out;
    hash.inp[1] <== commitment;
    hash.inp[2] <== secret_key;

    out <== hash.out;
}

template derive_public_key(){
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(2);
    component dst = NOMOS_KDF();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== secret_key;
    out <== hash.out;
}

template derive_unit(){
    signal input minting_covenant;
    signal input spending_covenant;
    signal input burning_covenant;
    signal output out;

    component hash = Poseidon2_hash(4);
    component dst = NOMOS_UNIT();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== minting_covenant;
    hash.inp[2] <== spending_covenant;
    hash.inp[3] <== burning_covenant;
    out <== hash.out;
}

