//test
pragma circom 2.1.9;

include "../../circom_circuits/hash/poseidon/poseidon_16_to_1_Jubjub.circom";
include "../../circom_circuits/hash/poseidon/poseidon_4_to_1_Jubjub.circom";

template coefficient_hash(){
    signal input coefficients[2048];

    signal output hash;

    component hasher[136];
    hasher[0] = permutation_16_to_1();
    hasher[0].in[0] <== 0;
    for(var i = 1; i<16; i++){
        hasher[0].in[i] <== coefficients[i-1];
    }


    for(var i = 1; i<136; i++){
        hasher[i] = permutation_16_to_1();
        hasher[i].in[0] <== hasher[i-1].out[0];
        for(var j = 1; j<16; j++){
            hasher[i].in[j] <== hasher[i-1].out[j] + coefficients[15*i+j-1];
        }
    }
    
    component final_hasher = hash_16_to_1();
    for (var i =0; i<8; i++){
        final_hasher.in[i] <== hasher[135].out[i] + coefficients[2040+i];
    }
    for (var i =8; i<16; i++){
        final_hasher.in[i] <== hasher[135].out[i];
    }

    hash <== final_hasher.out;
}

template drawn_random_point(){
    signal input da_commitment[2];
    signal input hash_of_data;

    signal output x_0;

    component final_hasher = hash_4_to_1();

    final_hasher.in[0] <== 0;
    final_hasher.in[1] <== da_commitment[0];
    final_hasher.in[2] <== da_commitment[1];
    final_hasher.in[3] <== hash_of_data;

    x_0 <== final_hasher.out;
}

template evaluate_polynomial(){
    signal input coefficients[2048];
    signal input evaluation_point;

    signal output result;

    signal intermediate_values[2046];
    intermediate_values[2045] <== coefficients[2047] * evaluation_point + coefficients[2046];
    for(var i = 2044; i >= 0; i--){
        intermediate_values[i] <== coefficients[i + 1] + intermediate_values[i+1] * evaluation_point;
    }

    result <== coefficients[0] + intermediate_values[0] * evaluation_point;
}

template proof_of_equivalence(){
    signal input coefficients[2048];
    signal input da_commitment[2];

    signal output x_0;
    signal output y_0;
    signal output coefficients_hash;

    //Hash of the coefficients
    component coefficient_hasher = coefficient_hash();
    for(var i = 0; i<2048; i++){
        coefficient_hasher.coefficients[i] <== coefficients[i];
    }
    coefficients_hash <== coefficient_hasher.hash;

    //Hash the coefficient hash with the da_commitment
    component point_drawer = drawn_random_point();
    point_drawer.da_commitment[0] <== da_commitment[0];
    point_drawer.da_commitment[1] <== da_commitment[1];
    point_drawer.hash_of_data <== coefficients_hash;
    x_0 <== point_drawer.x_0;

    //Evaluate the polynomial at x_0
    component evaluator = evaluate_polynomial();
    evaluator.evaluation_point <== x_0;
    for(var i =0; i<2048; i++){
        evaluator.coefficients[i] <== coefficients[i];
    }

    y_0 <== evaluator.result;
}


component main {public [da_commitment]} = proof_of_equivalence();