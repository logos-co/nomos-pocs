//test
pragma circom 2.1.9;

include "../../circom_circuits/hash/anemoi/anemoi_2_to_1_Jubjub.circom";
include "../../circom_circuits/hash/anemoi/anemoi_4_to_1_Jubjub.circom";
include "../../circom_circuits/hash/anemoi/anemoi_16_to_1_Jubjub.circom";
include "../../circom_circuits/circomlib/circuits/bitify.circom";
include "../../circom_circuits/circomlib/circuits/comparators.circom";
include "../../circom_circuits/Jubjub/escalarmulanyJubjub.circom";
include "../../circom_circuits/Jubjub/jubjub.circom";

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

template membership_checker(){
    signal input note_commitment;
    signal input pedersen_randomness;
    signal input pedersen_commitment[2];
    signal input h_curve_point[2];

    component note_commitment_bitifier = Num2Bits(255);
    component pedersen_randomness_bitifier = BLSNum2Bits_strict();
    note_commitment_bitifier.in <== note_commitment;
    pedersen_randomness_bitifier.in <== pedersen_randomness;

    // A is note_cm * G and B is r * H
    component A = EscalarMulAny(255);
    component B = EscalarMulAny(255);

    A.p[0] <== 0x11dafe5d23e1218086a365b99fbf3d3be72f6afd7d1f72623e6b071492d1122b;
    A.p[1] <== 0x1d523cf1ddab1a1793132e78c866c0c33e26ba5cc220fed7cc3f870e59d292aa;
    B.p[0] <== h_curve_point[0];
    B.p[1] <== h_curve_point[1];
    for(var i =0; i<255; i++){
        A.e[i] <== note_commitment_bitifier.out[i];
        B.e[i] <== pedersen_randomness_bitifier.out[i];
    }

    component pedersen = JubjubAdd();
    pedersen.x1 <== A.out[0];
    pedersen.y1 <== A.out[1];
    pedersen.x2 <== B.out[0];
    pedersen.y2 <== B.out[1];

    pedersen.xout === pedersen_commitment[0];
    pedersen.yout === pedersen_commitment[1];

}

template caulk_proof_of_validator(minimum_stake){   //TODO: put minimum_stake in the input to change it dynamically
    signal input pedersen_commitment[2];
    signal input h_curve_point[2];

        // Note variables
    signal input constraints;    // Every note field represented as F_p elements for now (constraints are represented by their Merkle root)
    signal input value;          // 0 if no more notes needed
    signal input unit;
    signal input state;          // This field hold the identity of the owner (its public key or ID) and is revealed
    signal input note_nonce;
    signal input nullifier_secret_key;

    signal input pedersen_randomness;

    signal output nullifiers;
    signal output updated_commiments;

        // Compute the note commitments
    component note_committer = commitment_computer();
    note_committer.note_nonce <== note_nonce;
    note_committer.nullifier_public_key <== nullifier_secret_key;        // TODO: reflect the nullifier public key computation later when defined
    note_committer.value <== value;
    note_committer.constraints <== constraints;
    note_committer.unit <== unit;
    note_committer.state <== state;

        // Check the commitments membership
    component membership_checker = membership_checker();
    membership_checker.note_commitment <== note_committer.commitment;
    membership_checker.pedersen_randomness <== pedersen_randomness;
    membership_checker.pedersen_commitment[0] <== pedersen_commitment[0];
    membership_checker.pedersen_commitment[1] <== pedersen_commitment[1];
    membership_checker.h_curve_point[0] <== h_curve_point[0];
    membership_checker.h_curve_point[1] <== h_curve_point[1];

        // Check that the value exceed the minimum stake
    component isLess = BLSLessThan(253);
    isLess.in[0] <== minimum_stake;
    isLess.in[1] <== value;
    isLess.out === 1;

    // Compute the note nullifiers
    component nullifier_computer = nullifier_computer();
    nullifier_computer.note_nonce <== note_nonce;
    nullifier_computer.nullifier_secret_key <== nullifier_secret_key;
    nullifier_computer.value <== value;
    nullifiers <== nullifier_computer.nullifier;

        // Compute the evolved nonces
    component nonce_updater = nonce_updater();
    nonce_updater.note_nonce <== note_nonce;
    nonce_updater.nullifier_secret_key <== nullifier_secret_key;

        // Compute the new note commitments
    component updated_note_committer = commitment_computer();
    updated_note_committer.note_nonce <== nonce_updater.updated_nonce;
    updated_note_committer.nullifier_public_key <== nullifier_secret_key;    // TODO: reflect the nullifier public key computation later when defined
    updated_note_committer.value <== value;
    updated_note_committer.constraints <== constraints;
    updated_note_committer.unit <== unit;
    updated_note_committer.state <== state;
    updated_commiments <== updated_note_committer.commitment;

}


component main {public [state,pedersen_commitment,h_curve_point]} = caulk_proof_of_validator(10000);