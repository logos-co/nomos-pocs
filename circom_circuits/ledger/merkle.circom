//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../circomlib/circuits/comparators.circom";

// proof of Merkle membership of depth n
//                  /!\ To call this function, it's important to check that each selector is a bit before!!!
template proof_of_membership(n) {
    signal input nodes[n];      // The path forming the Merkle proof
    signal input selector[n];   // it's the leaf's indice in big endian bits 
    signal input root;
    signal input leaf;
    signal output out;
    

    component compression_hash[n];

    compression_hash[0] = Poseidon2_hash(2);
    compression_hash[0].inp[0] <== leaf - selector[n-1] * (leaf - nodes[0]);
    compression_hash[0].inp[1] <== nodes[0] - selector[n-1] * (nodes[0] - leaf);

    for(var i=1; i<n; i++){
        compression_hash[i] = Poseidon2_hash(2);
        compression_hash[i].inp[0] <== compression_hash[i-1].out - selector[n-1-i] * (compression_hash[i-1].out - nodes[i]);
        compression_hash[i].inp[1] <== nodes[i] - selector[n-1-i] * (nodes[i] - compression_hash[i-1].out);
    }

    component eq = IsEqual();
    eq.in[0] <== root;
    eq.in[1] <== compression_hash[n-1].out;

    out <== eq.out;
}