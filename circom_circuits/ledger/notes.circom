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
    // int.from_bytes(hashlib.sha256(b"NOMOS_NOTE_CM").digest()[:-1], "little") = 181645510297841241569044198526601622686169271532834574969543446901055041748
    hash.inp[0] <== 181645510297841241569044198526601622686169271532834574969543446901055041748;
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
    // int.from_bytes(hashlib.sha256(b"NOMOS_NOTE_NF").digest()[:-1], "little") = 310945536431723660304787929213143698356852257431717126117833288836338828411
    hash.inp[0] <==  310945536431723660304787929213143698356852257431717126117833288836338828411;
    hash.inp[1] <== commitment;
    hash.inp[2] <== secret_key;

    out <== hash.out;
}

template derive_public_key(){
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(2);
    // int.from_bytes(hashlib.sha256(b"NOMOS_KDF").digest()[:-1], "little") = 355994159511987982411097843485998670968942801951585260613801918349630142543
    hash.inp[0] <== 355994159511987982411097843485998670968942801951585260613801918349630142543;
    hash.inp[1] <== secret_key;
    out <== hash.out;
}

