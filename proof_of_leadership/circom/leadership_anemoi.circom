//test
pragma circom 2.1.9;

include "../../circom_circuits/hash/anemoi/anemoi_2_to_1_Jubjub.circom";
include "../../circom_circuits/hash/anemoi/anemoi_4_to_1_Jubjub.circom";
include "../../circom_circuits/hash/anemoi/anemoi_16_to_1_Jubjub.circom";
include "../../circom_circuits/circomlib/circuits/bitify.circom";

template BLSLessThan(n) {
    assert(n <= 253);
    signal input in[2];
    signal output out;

    component n2b = Num2Bits(n+1);

    n2b.in <== in[0]+ (1<<n) - in[1];

    out <== 1-n2b.out[n];
}

template BLSNum2Bits_strict() {
    signal input in;
    signal output out[255];

    component check_range = CompConstant(23487852865797141623554994256013988874373056334117496812739262697960298774528); // -1 - 2**254 (p-1 without its first bit)
    component n2b = Num2Bits(255);
    in ==> n2b.in;

    for (var i=0; i<255; i++) {
        n2b.out[i] ==> out[i];
        if(i != 0){
            n2b.out[i] ==> check_range.in[i-1];
        }
    }

    check_range.out * (n2b.out[0]) === 0; //must be zero exept if the first bit is 0 => then in is on 254 bits and p-1 on 255
}

template BLSBits2Num_strict() {
    signal input in[255];
    signal output out;

    component check_range = CompConstant(23487852865797141623554994256013988874373056334117496812739262697960298774528);
    component b2n = Bits2Num(255);

    for (var i=0; i<255; i++) {
        in[i] ==> b2n.in[i];
        if(i != 0){
            in[i] ==> check_range.in[i-1];
        }
    }

    check_range.out * in[0] === 0;

    b2n.out ==> out;
}

template check_bits(n){
    signal input bits[n];
    for(var i=0; i<n; i++){
        bits[i] * (1-bits[i]) === 0;
    }
}

template check_lottery(){
    signal input epoch_nonce;
    signal input slot_number;
    signal input t0;
    signal input t1;        // The precomputed threshold values

    signal input constraints;
    signal input value;
    signal input unit;
    signal input state;
    signal input note_nonce;
    signal input nullifier_secret_key;
    signal input randomness;

    component hash = hash_16_to_1();

            //The b"lead" Tag converted in F_p element (from bits with big endian order)
    hash.in[0] <== 1818583396;

    hash.in[1] <== epoch_nonce;
    hash.in[2] <== slot_number;
    hash.in[3] <== constraints;
    hash.in[4] <== value;
    hash.in[5] <== unit;
    hash.in[6] <== state;
    hash.in[7] <== note_nonce;
    hash.in[8] <== nullifier_secret_key;
    hash.in[9] <== randomness;
    for(var i=10; i<16; i++){
        hash.in[i] <== 0;
    }

            // As p-1 is divisible by 4, if X follows a uniform distribution between 0 and p-1
            // Y = X % (p-1)/4 nearly follows a uniform distribution between 0 and ((p-1)/4)-1 (exept that 0 has 20% more chance to appear than another element which is negligible)
            // So we transform the hash of a maximum of 255 to a hash of a maximum of 253 bits by taking its modulo
            // (p-1)/4 = 13108968793781547619861935127046491459422638125131909455650914674984645296128
            // TODO: check this part to ensure it is secure
    signal quotient;
    signal ticket;
    quotient <-- hash.out \ 13108968793781547619861935127046491459422638125131909455650914674984645296128;
    ticket <-- hash.out % 13108968793781547619861935127046491459422638125131909455650914674984645296128;

            //check that quotient is 0,1,2 or 3
    signal check_quotient[2];
    check_quotient[0] <== quotient * (1-quotient);
    check_quotient[1] <== (2-quotient)*(3-quotient);
    check_quotient[0] * check_quotient[1] === 0;

            //check the correctness of the division
    ticket + quotient * 13108968793781547619861935127046491459422638125131909455650914674984645296128 === hash.out;

            //check that the ticket is less than the divisor
    component isLess = CompConstant(13108968793781547619861935127046491459422638125131909455650914674984645296128);
    component bitifier = BLSNum2Bits_strict();

    bitifier.in <== ticket;
    bitifier.out[254] === 0;
    for(var i=0; i<254; i++){
        isLess.in[i] <== bitifier.out[i];
    }
    isLess.out === 0;


    
            // Compute the threshold
    signal intermediate_value;
    signal threshold;
    intermediate_value <== t0 + t1 * value;
    threshold <== intermediate_value * value;

            // Ensure that the ticket is winning
    component isLess2 = BLSLessThan(253);

    isLess2.in[0] <== ticket;
    isLess2.in[1] <== threshold;
    isLess2.out === 1;
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

template commitment_computer(){     // TODO: ensure all field are hash
    signal input note_nonce;
    signal input nullifier_public_key;
    signal input value;
    signal output commitment;

    component hash = hash_4_to_1();

            //The b"coin-commitment" Tag converted in F_p element (from bits with big endian order)
    hash.in[0] <== 516297089516239580383111224192495220;
    hash.in[1] <== note_nonce;
    hash.in[2] <== nullifier_public_key;
    hash.in[3] <== value;

    commitment <== hash.out;
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

    root === hash[31].out;

}

template anemoi_proof_of_leadership(){
    signal input epoch_nonce;   //F_p (BLS12-381 scalar field)
    signal input slot_number;   //F_p (BLS12-381 scalar field)
    signal input t0;            // Precomputed threshold elements in F_p
    signal input t1;
    signal input commitments_root;

        // Note variables
    signal input constraints;   // Every note field represented as F_p elements for now (constraints are represented by their Merkle root)
    signal input value;
    signal input unit;
    signal input state;
    signal input note_nonce;
    signal input nullifier_secret_key;
    signal input randomness;
    signal input index[32];     //Position of the note commitment in bits in big endian
    signal input nodes[32];     //Merkle proof of the commitment

    signal output nullifier;
    signal output updated_commiment;


        // Check that index inputs are indeed bits
    component bit_checker = check_bits(32);
    for(var i=0; i<32; i++){
        bit_checker.bits[i] <== index[i];
    }

        // Check that r < threshold
    component lottery_checker = check_lottery();
    lottery_checker.epoch_nonce <== epoch_nonce;
    lottery_checker.slot_number <== slot_number;
    lottery_checker.t0 <== t0;
    lottery_checker.t1 <== t1;
    lottery_checker.constraints <== constraints;
    lottery_checker.value <== value;
    lottery_checker.unit <== unit;
    lottery_checker.state <== state;
    lottery_checker.note_nonce <== note_nonce;
    lottery_checker.nullifier_secret_key <== nullifier_secret_key;
    lottery_checker.randomness <== randomness;


        // Compute the note commitment
    component note_committer = commitment_computer();
    note_committer.note_nonce <== note_nonce;
    note_committer.nullifier_public_key <== nullifier_secret_key;        // TODO: reflect the nullifier public key computation later when defined
    note_committer.value <== value;

        // Check the commitment membership
    component membership_checker = membership_checker();
    membership_checker.leaf <== note_committer.commitment;
    membership_checker.root <== commitments_root;
    for(var i =0; i<32; i++){
        membership_checker.index[i] <== index[i];
        membership_checker.node[i] <== nodes[i];
    }


        // Compute the note nullifier
    component nullifier_computer = nullifier_computer();
    nullifier_computer.note_nonce <== note_nonce;
    nullifier_computer.nullifier_secret_key <== nullifier_secret_key;
    nullifier_computer.value <== value;
    nullifier <== nullifier_computer.nullifier;


        // Compute the evolved nonce
    component nonce_updater = nonce_updater();
    nonce_updater.note_nonce <== note_nonce;
    nonce_updater.nullifier_secret_key <== nullifier_secret_key;


        // Compute the new note commitment
    component updated_note_committer = commitment_computer();
    updated_note_committer.note_nonce <== nonce_updater.updated_nonce;
    updated_note_committer.nullifier_public_key <== nullifier_secret_key;    // TODO: reflect the nullifier public key computation later when defined
    updated_note_committer.value <== value;
    updated_commiment <== updated_note_committer.commitment;
    
}


component main {public [epoch_nonce, slot_number, t0, t1, commitments_root]} = anemoi_proof_of_leadership();