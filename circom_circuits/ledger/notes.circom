//test
pragma circom 2.1.9;

include "poseidon2_hash.circom";
include "merkle.circom";

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
    hash.inp[0] <== 78797779839578798469956777;        //78797779839578798469956777 = NOMOS_NOTE_CM in ASCII
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
    hash.inp[0] <==  78797779839578798469957870;    //78797779839578798469957870 = NOMOS_NOTE_NF in ASCII
    hash.inp[1] <== commitment;
    hash.inp[2] <== secret_key;

    out <== hash.out;
}

template derive_public_key(){
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(2);
    hash.inp[0] <== 787977798395756870;         // 787977798395756870 = NOMOS_KDF in ASCII
    hash.inp[1] <== secret_key;
    out <== hash.out;
}

