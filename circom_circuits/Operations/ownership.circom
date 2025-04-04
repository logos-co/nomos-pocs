//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template proof_of_private_note_ownership(nInput){
    signal input state[nInput];
    signal input value[nInput];
    signal input nonce[nInput];
    signal input zoneID[nInput];
    signal input secret_key[nInput];
    signal input minting_covenant[nInput];
    signal input transfer_covenant[nInput];
    signal input burning_covenant[nInput];

    signal input attached_data;

    signal output commitment[nInput];

    component pk[nInput];
    for(var i =0; i<nInput; i++){
        pk[i] = derive_public_key();
        pk[i].secret_key <== secret_key[i];
    }

    component unit[nInput];
    for(var i=0; i< nInput; i++){
        unit[i] = derive_unit();
        unit[i].minting_covenant <== minting_covenant[i];
        unit[i].transfer_covenant <== transfer_covenant[i];
        unit[i].burning_covenant <== burning_covenant[i];
    }

    component cm[nInput];
    for(var i =0; i< nInput; i++){
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

template proof_of_public_note_ownership(nInput){
    signal input secret_key[nInput];

    signal input attached_data;

    signal output public_key[nInput];

    component pk[nInput];
    for(var i =0; i<nInput; i++){
        pk[i] = derive_public_key();
        pk[i].secret_key <== secret_key[i];
        public_key[i] <== pk[i].out;
    }

    // dummy constraint to avoid unused public input to be erased after compilation optimisation
    signal dummy;
    dummy <== attached_data * attached_data;
}

component main {public [attached_data]}= proof_of_public_note_ownership(1);