//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../misc/comparator.circom";

// proof of Merkle membership of depth n
//                  /!\ To call this function, it's important to check that each selector is a bit before!!!
template proof_of_membership(n) {
    signal input nodes[n];      // The path forming the Merkle proof
    signal input selector[n];   // it's the leaf's indice in big endian bits 
    signal input root;
    signal input leaf;
    

    component compression_hash[n];

    compression_hash[0] = Poseidon2_hash(2);
    compression_hash[0].inp[0] <== leaf - selector[n-1] * (leaf - nodes[0]);
    compression_hash[0].inp[1] <== nodes[0] - selector[n-1] * (nodes[0] - leaf);

    for(var i=1; i<n; i++){
        compression_hash[i] = Poseidon2_hash(2);
        compression_hash[i].inp[0] <== compression_hash[i-1].out - selector[n-1-i] * (compression_hash[i-1].out - nodes[i]);
        compression_hash[i].inp[1] <== nodes[i] - selector[n-1-i] * (nodes[i] - compression_hash[i-1].out);
    }

    root === compression_hash[n-1].out;
}


//                  /!\ DEPRECATED  /!\
// proof of Merkle non-membership using an IMT of depth n
//                  /!\ To call this function, it's important to check that each selector is a bit before!!!
template proof_of_non_membership(n) {
    signal input previous;    // We prove that the nullifier isn't in the set because it falls between previous and next
    signal input nullifier;
    signal input next;
    signal input nodes[n];
    signal input selector[n];
    signal input root;

    component hash = Poseidon2_hash(2);
    component comparator[2];
    component membership = proof_of_membership(n);

    // Recover the leaf representing previous pointing to next in the IMT
    hash.inp[0] <== previous; 
    hash.inp[1] <== next;

    // Verify that the leaf computed is indeed in the IMT
    membership.root <== root;
    membership.leaf <== hash.out;
    for(var i =0; i < n; i++){
        membership.nodes[i] <== nodes[i];
        membership.selector[i] <== selector[i];
    }

    // Check that nullifier stictly falls between previous and next.
    comparator[0] = SafeFullLessThan();
    comparator[0].a <== previous;
    comparator[0].b <== nullifier;
    comparator[0].out === 1;

    comparator[1] = SafeFullLessThan();
    comparator[1].a <== nullifier;
    comparator[1].b <== next;
    comparator[1].out === 1;
}