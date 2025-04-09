//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template shielded_transaction(maxInputs, maxOutputs){

    signal input minting_covenant;      // Used to derive the unit and make sure the token use a no-op spending covenant.
    signal input burning_covenant;

    //consummed notes
        // notes themselves
    signal input state_in[maxInputs];
    signal input value_in[maxInputs];
    signal input nonce_in[maxInputs];
    signal input secret_key_in[maxInputs];
    signal input zoneID_in[maxInputs];
        // proof of commitment membership
    signal input cm_nodes[maxInputs][32];
    signal input cm_selectors[maxInputs][32];         // must be bits
    signal input commitments_root[maxInputs];
    signal input is_a_input_note[maxInputs];              // Selector to say if note i is a real entry or a dummy input, must be 0 or 1
    signal output nullifier[maxInputs];                   // /!\ It needs to be checked outside the circuit by the validators
                                                          // Padding can be realized by repeating the same note on selector 0 (validators will ignore the nullifier)

    //created notes
    signal input state_out[maxOutputs];
    signal input value_out[maxOutputs];
    signal input nonce_out[maxOutputs];
    signal input public_key_out[maxOutputs];
    signal input zoneID_out[maxOutputs];
    signal input is_a_output_note[maxOutputs];         // Selector to say if note i is a real output or a dummy output, must be 0 or 1
    signal output commitments[maxOutputs];              // /!\ It needs to be checked outside the circuit by the validators
                                                        // Padding can be realized by repeating the same note on selector 0 (validators will ignore the commitment)
    signal input attached_data;

    signal output balance;
    signal output unit;         // Disclose the unit of the transaction

    //Derive the unit
    component derive_unit = derive_unit();
    derive_unit.minting_covenant <== minting_covenant;
    derive_unit.spending_covenant <== 0;                   // 0 encodes the fact that it's a no-op transfer covenant
    derive_unit.burning_covenant <== burning_covenant;
    unit <== derive_unit.out;


    // Verify the ownership of the consummed notes deriving the public keys from the secret keys
    component pk[maxInputs];
    for(var i =0; i<maxInputs; i++){
        pk[i] = derive_public_key();
        pk[i].secret_key <== secret_key_in[i];
    }


    // Derive the commitments of the consummed notes
    component cm_in[maxInputs];
    for(var i =0; i<maxInputs; i++){
        cm_in[i] = commitment();
        cm_in[i].state <== state_in[i];
        cm_in[i].value <== value_in[i];
        cm_in[i].unit <== unit;
        cm_in[i].nonce <== nonce_in[i];
        cm_in[i].zoneID <== zoneID_in[i];
        cm_in[i].public_key <== pk[i].out;
    }

    // Derive the nullifiers of the consummed notes
    component nf[maxInputs];
    for(var i=0; i<maxInputs; i++){
        nf[i] = nullifier();
        nf[i].commitment <== cm_in[i].out;
        nf[i].secret_key <== secret_key_in[i];
        nullifier[i] <== nf[i].out;
    }


    // Prove the commitment membership against the chosen root(s)
    component cm_membership[maxInputs];
    for(var i =0; i< maxInputs; i++){
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
    component cm_out[maxOutputs];
    for(var i =0; i<maxOutputs; i++){
        cm_out[i] = commitment();
        cm_out[i].state <== state_out[i];
        cm_out[i].value <== value_out[i];
        cm_out[i].unit <== unit;
        cm_out[i].nonce <== nonce_out[i];
        cm_out[i].zoneID <== zoneID_out[i];
        cm_out[i].public_key <== public_key_out[i];
        commitments[i] <== cm_out[i].out;
    }

    signal b[maxInputs + maxInputs];
    b[0] <== value_in[0] * is_a_input_note[0];
    for(var i = 1; i< maxInputs; i++){
        b[i] <== b[i-1] + value_in[i] * is_a_input_note[i];
    }
    for(var i =0; i< maxOutputs; i++){
        b[i + maxInputs] <== b[maxInputs + i - 1] - value_out[i] * is_a_output_note[i];
    }
    balance <== b[maxInputs + maxOutputs - 1];

    //dummy quadratic contraints to avoid optimisation erasing the public input
    signal dummy;
    dummy <== attached_data * attached_data;

}

component main {public [zoneID_in,is_a_input_note,is_a_output_note,commitments_root,zoneID_out]}= shielded_transaction(4,4);