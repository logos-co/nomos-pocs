//test
pragma circom 2.1.9;

include "../../circom_circuits/hash/poseidon/poseidon_2_to_1_Jubjub.circom";
include "../../circom_circuits/hash/poseidon/poseidon_4_to_1_Jubjub.circom";
include "../../circom_circuits/hash/poseidon/poseidon_16_to_1_Jubjub.circom";
include "../../circom_circuits/circomlib/circuits/bitify.circom";
include "../../circom_circuits/circomlib/circuits/comparators.circom";

template BLSLessThan(n) {
    assert(n <= 253);
    signal input in[2];
    signal output out;

    component n2b = Num2Bits(n+1);

    n2b.in <== in[0]+ (1<<n) - in[1];

    out <== 1-n2b.out[n];
}

template check_bits(n){
    signal input bits[n];
    for(var i=0; i<n; i++){
        bits[i] * (1-bits[i]) === 0;
    }
}

template commitment_computer(){
    signal input note_nonce;
    signal input nullifier_public_key;
    signal input value;
    signal input constraints;
    signal input unit;
    signal input state;
    signal output commitment;

    component hash = hash_16_to_1();

            //The b"coin-commitment" Tag converted in F_p element (from bits with big endian order)
    hash.in[0] <== 516297089516239580383111224192495220;
    hash.in[1] <== note_nonce;
    hash.in[2] <== nullifier_public_key;
    hash.in[3] <== value;
    hash.in[4] <== constraints;
    hash.in[5] <== unit;
    hash.in[6] <== state;
    for(var i=7; i<16; i++){
        hash.in[i] <== 0;
    }

    commitment <== hash.out;
}

template nullifier_computer(){
    signal input note_nonce;
    signal input nullifier_secret_key;
    signal input value;
    signal output nullifier;

    component hash = hash_4_to_1();

            //The b"coin-nullifier" Tag converted in F_p element (from bits with big endian order)
    hash.in[0] <== 2016785505923014207119328528655730;
    hash.in[1] <== note_nonce;
    hash.in[2] <== nullifier_secret_key;
    hash.in[3] <== value;

    nullifier <== hash.out;
}

template nonce_updater(){
    signal input note_nonce;
    signal input nullifier_secret_key;
    signal output updated_nonce;

    component hash = hash_4_to_1();

            //The b"coin-evolve" Tag converted in F_p element (from bits with big endian order)
    hash.in[0] <== 120209783668687835891529317;
    hash.in[1] <== note_nonce;
    hash.in[2] <== nullifier_secret_key;
    hash.in[3] <== 0;

    updated_nonce <== hash.out;
}

template membership_checker(){
    signal input leaf;                      //The note commitment
    signal input root;                      //The root of the Merkle Tree (of depth 32)
    signal input index[32];                 //Position of the note commitment in bits in big endian
    signal input node[32];                  //Complementary hashes
    signal input is_null;                   //If is_null is 1 we don't check the membership (any value of node and index will be correct)

    component hash[32];

    for(var i=0; i<32; i++){
        hash[i] = hash_2_to_1();
    }

    hash[0].in[0] <== leaf - index[31] * (leaf - node[0]);
    hash[0].in[1] <== node[0] - index[31] * (node[0] - leaf);

    for(var i=1; i<32; i++){
        hash[i].in[0] <== hash[i-1].out - index[31-i] * (hash[i-1].out - node[i]);
        hash[i].in[1] <== node[i] - index[31-i] * (node[i] - hash[i-1].out);
    }

    root === hash[31].out * (1 - is_null);

}

template poseidon_proof_of_validator(max_notes, minimum_stake){     //TODO: put minimum_stake in the input to change it dynamically
    signal input commitments_root;

        // Note variables
    signal input constraints[max_notes];    // Every note field represented as F_p elements for now (constraints are represented by their Merkle root)
    signal input value[max_notes];          // 0 if no more notes needed
    signal input unit[max_notes];
    signal input state[max_notes];          // This field hold the identity of the owner (its public key or ID)
    signal input note_nonce[max_notes];
    signal input nullifier_secret_key[max_notes];
    signal input index[max_notes][32];     //Position of the note commitment in bits in big endian
    signal input nodes[max_notes][32];     //Merkle proof of the commitment

    signal output identity;
    signal output nullifiers[max_notes];
    signal output updated_commiments[max_notes];


        // Check that index inputs are indeed bits
    component bit_checker[max_notes];
    for(var i=0; i<max_notes; i++){
        bit_checker[i] = check_bits(32);
        for(var j=0; j<32; j++){
            bit_checker[i].bits[j] <== index[i][j];
        }
    }

        // Compute the note commitments
    component note_committer[max_notes];
    for(var i=0; i<max_notes; i++){
        note_committer[i] = commitment_computer();
        note_committer[i].note_nonce <== note_nonce[i];
        note_committer[i].nullifier_public_key <== nullifier_secret_key[i];        // TODO: reflect the nullifier public key computation later when defined
        note_committer[i].value <== value[i];
        note_committer[i].constraints <== constraints[i];
        note_committer[i].unit <== unit[i];
        note_committer[i].state <== state[i];
    }

        //check the identity between the notes
    identity <== state[0];       // The first note must not be null
    component is_null[max_notes];
    is_null[0] = IsZero();
    is_null[0].in <== value[0];
    is_null[0].out === 0;
    signal intermediate[max_notes-1];
    for(var i=1; i<max_notes; i++){
        is_null[i] = IsZero();
        is_null[i].in <== value[i];
        intermediate[i-1] <== identity * (1 - is_null[i].out);
        intermediate[i-1] === state[i] * (1 - is_null[i].out);
    }

        // Check the commitments membership
    component membership_checker[max_notes];
    for(var i=0; i<max_notes; i++){
        membership_checker[i] = membership_checker();
        membership_checker[i].leaf <== note_committer[i].commitment;
        membership_checker[i].root <== commitments_root * (1- is_null[i].out);      // Set the root at 0 is note is null
        membership_checker[i].is_null <== is_null[i].out;
        for(var j =0; j<32; j++){
            membership_checker[i].index[j] <== index[i][j];
            membership_checker[i].node[j] <== nodes[i][j];
        }
    }

        // Check that the value exceed the minimum stake
    signal sum[max_notes-1];
    sum[0] <== value[0] + value[1];
    for(var i = 1; i<max_notes-1; i++){
        sum[i] <== sum[i-1] + value[i+1];
    }
    component isLess = BLSLessThan(253);
    isLess.in[0] <== minimum_stake;
    isLess.in[1] <== sum[max_notes-2];
    isLess.out === 1;


    // Compute the note nullifiers
    component nullifier_computer[max_notes];
    for(var i=0; i<max_notes; i++){
        nullifier_computer[i] = nullifier_computer();
        nullifier_computer[i].note_nonce <== note_nonce[i];
        nullifier_computer[i].nullifier_secret_key <== nullifier_secret_key[i];
        nullifier_computer[i].value <== value[i];
        nullifiers[i] <== nullifier_computer[i].nullifier;
    }

        // Compute the evolved nonces
    component nonce_updater[max_notes];
    for(var i=0; i<max_notes; i++) {
        nonce_updater[i] = nonce_updater();
        nonce_updater[i].note_nonce <== note_nonce[i];
        nonce_updater[i].nullifier_secret_key <== nullifier_secret_key[i];
    } 


        // Compute the new note commitments
    component updated_note_committer[max_notes];
    for(var i=0; i<max_notes; i++) {
        updated_note_committer[i] = commitment_computer();
        updated_note_committer[i].note_nonce <== nonce_updater[i].updated_nonce;
        updated_note_committer[i].nullifier_public_key <== nullifier_secret_key[i];    // TODO: reflect the nullifier public key computation later when defined
        updated_note_committer[i].value <== value[i];
        updated_note_committer[i].constraints <== constraints[i];
        updated_note_committer[i].unit <== unit[i];
        updated_note_committer[i].state <== state[i];
        updated_commiments[i] <== updated_note_committer[i].commitment;
    }

}


component main {public [commitments_root]} = poseidon_proof_of_validator(50,10000);