//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template proof_of_unshielded_note_ownership(maxInput){
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

component main {public [attached_data]}= proof_of_unshielded_note_ownership(1);