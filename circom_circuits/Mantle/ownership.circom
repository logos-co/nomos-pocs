//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template proof_of_private_note_ownership(maxInput){
    signal input state[maxInput];
    signal input value[maxInput];
    signal input nonce[maxInput];
    signal input zoneID[maxInput];
    signal input secret_key[maxInput];
    signal input minting_covenant[maxInput];
    signal input spending_covenant[maxInput];
    signal input burning_covenant[maxInput];

    signal input attached_data;

    signal output commitment[maxInput];

    component pk[maxInput];
    for(var i =0; i<maxInput; i++){
        pk[i] = derive_public_key();
        pk[i].secret_key <== secret_key[i];
    }

    component unit[maxInput];
    for(var i=0; i< maxInput; i++){
        unit[i] = derive_unit();
        unit[i].minting_covenant <== minting_covenant[i];
        unit[i].spending_covenant <== spending_covenant[i];
        unit[i].burning_covenant <== burning_covenant[i];
    }

    component cm[maxInput];
    for(var i =0; i< maxInput; i++){
        cm[i] = commitment();
        cm[i].state <== state[i];
        cm[i].value <== value[i];
        cm[i].unit <== unit[i].out;
        cm[i].nonce <== nonce[i];
        cm[i].zoneID <== zoneID[i];
        cm[i].public_key <== pk[i].out;
    }

    // dummy constraint to avoid unused public input to be erased after compilation optimisation
    signal dummy;
    dummy <== attached_data * attached_data;
}

template proof_of_public_note_ownership(maxInput){
    signal input secret_key[maxInput];

    signal input attached_data;

    signal output public_key[maxInput];

    component pk[maxInput];
    for(var i =0; i<maxInput; i++){
        pk[i] = derive_public_key();
        pk[i].secret_key <== secret_key[i];
        public_key[i] <== pk[i].out;
    }

    // dummy constraint to avoid unused public input to be erased after compilation optimisation
    signal dummy;
    dummy <== attached_data * attached_data;
}

component main {public [attached_data]}= proof_of_public_note_ownership(1);