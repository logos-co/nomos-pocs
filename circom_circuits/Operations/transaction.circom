//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template transfer(nInputs, nOutputs){

    //consummed notes
        // notes themselves
    signal input state_in[nInputs];
    signal input value_in[nInputs];
    signal input nonce_in[nInputs];
    signal input secret_key_in[nInputs];
    signal input zoneID_in[nInputs];
        // proof of commitment membership
    signal input cm_nodes[nInputs][32];
    signal input cm_selectors[nInputs][32];         // must be bits
    signal input commitments_root[nInputs];
    signal output nullifier[nInputs];

    //created notes
    signal input state_out[nOutputs];
    signal input value_out[nOutputs];
    signal input nonce_out[nOutputs];
    signal input public_key_out[nOutputs];
    signal input zoneID_out[nOutputs];
    signal output commitments[nOutputs];

    signal input attached_data;

    signal output balance;


    // Verify the ownership of the consummed notes deriving the public keys from the secret keys
    component pk[nInputs];
    for(var i =0; i<nInputs; i++){
        pk[i] = derive_public_key();
        pk[i].secret_key <== secret_key_in[i];
    }


    // Derive the commitments of the consummed notes
    component nmo = NMO(); // NMO token constant
    component cm_in[nInputs];
    for(var i =0; i<nInputs; i++){
        cm_in[i] = commitment();
        cm_in[i].state <== state_in[i];
        cm_in[i].value <== value_in[i];
        cm_in[i].unit <== nmo.out;
        cm_in[i].nonce <== nonce_in[i];
        cm_in[i].zoneID <== zoneID_in[i];
        cm_in[i].public_key <== pk[i].out;
    }


    // Derive the nullifiers of the consummed notes
    component nf[nInputs];
    for(var i=0; i<nInputs; i++){
        nf[i] = nullifier();
        nf[i].commitment <== cm_in[i].out;
        nf[i].secret_key <== secret_key_in[i];
        nullifier[i] <== nf[i].out;
    }


    // Prove the commitment membership against the chosen root(s)
    component cm_membership[nInputs];
    for(var i =0; i< nInputs; i++){
            //First check selectors are indeed bits
        for(var j = 0; j < 32; j++){
            cm_selectors[i][j] * (1 - cm_selectors[i][j]) === 0;
        }
            //Then check the proof of membership
        cm_membership[i] = proof_of_membership(32);
        for(var j = 0; j < 32; j++){
            cm_membership[i].nodes[j] <== cm_nodes[i][j];
            cm_membership[i].selector[j] <== cm_selectors[i][j];
        }
        cm_membership[i].root <== commitments_root[i];
        cm_membership[i].leaf <== cm_in[i].out;
    }


    // Derive the commitments of the created notes
    component cm_out[nOutputs];
    for(var i =0; i<nOutputs; i++){
        cm_out[i] = commitment();
        cm_out[i].state <== state_out[i];
        cm_out[i].value <== value_out[i];
        cm_out[i].unit <== nmo.out;
        cm_out[i].nonce <== nonce_out[i];
        cm_out[i].zoneID <== zoneID_out[i];
        cm_out[i].public_key <== public_key_out[i];
        commitments[i] <== cm_out[i].out;
    }

    var b = 0;
    for(var i = 0; i< nInputs; i++){
        b += value_in[i];
    }
    for(var i =0; i< nOutputs; i++){
        b -= value_out[i];
    }
    balance <== b;

    //dummy quadratic contraints to avoid optimisation erasing the public input
    signal dummy;
    dummy <== attached_data * attached_data;

}

component main {public [zoneID_in,commitments_root,zoneID_out]}= transfer(1,1);