//test
pragma circom 2.1.9;

include "../circomlib/circuits/bitify.circom";
include "../circomlib/circuits/comparators.circom";

// int.from_bytes(b"LEAD_V1", byteorder="big") = 21468244852299313
template LEAD_V1(){
    signal output out;
    out <== 21468244852299313;
}


// int.from_bytes(b"NOMOS_POL_SK_V1", byteorder="big") = 406607590443025360526585251810465329
template NOMOS_POL_SK_V1(){
    signal output out;
    out <== 406607590443025360526585251810465329;
}


// int.from_bytes(b"NOMOS_NONCE_CONTRIB_V1", byteorder="big") = 29299164684883585569547934353856711107288148897388081
template NOMOS_NONCE_CONTRIB_V1(){
    signal output out;
    out <== 29299164684883585569547934353856711107288148897388081;
}


// int.from_bytes(b"NOMOS_KDF", byteorder="big") = 1444560348471047701574
template NOMOS_KDF(){
    signal output out;
    out <== 1444560348471047701574;
}


// int.from_bytes(b"NOMOS_NOTE_ID_V1", byteorder="big") = 104091543153414482850642014312194856497
template NOMOS_NOTE_ID_V1(){
    signal output out;
    out <== 104091543153414482850642014312194856497;
}


// int.from_bytes(b"SELECTION_RANDOMNESS_V1", byteorder="big") = 7975748052709904163696334751877473705917106215133861425
template SELECTION_RANDOMNESS_V1(){
    signal output out;
    out <== 7975748052709904163696334751877473705917106215133861425;
}


// int.from_bytes(b"KEY_NULLIFIER_V1", byteorder="big") = 100052180852480707195751331170348914225
template KEY_NULLIFIER_V1(){
    signal output out;
    out <== 100052180852480707195751331170348914225;
}


// int.from_bytes(b"REWARD_VOUCHER", byteorder="big") = 1668651334877449245987336926807378
template REWARD_VOUCHER(){
    signal output out;
    out <== 1668651334877449245987336926807378;
}


// int.from_bytes(b"VOUCHER_NF", byteorder="big") = 407586954142391778364998
template VOUCHER_NF(){
    signal output out;
    out <== 407586954142391778364998;
}