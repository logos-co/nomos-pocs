//test
pragma circom 2.1.9;

include "../../circom_circuits/circomlib/circuits/sha256/sha256.circom";
include "../../circom_circuits/circomlib/circuits/bitify.circom";

template BLSLessThan(n) {
    assert(n <= 253);
    signal input in[2];
    signal output out;

    component n2b = Num2Bits(n+1);

    n2b.in <== in[0]+ (1<<n) - in[1];

    out <== 1-n2b.out[n];
}

template lottery_ticket() {
    signal input epoch_nonce[256];   //F_p (BLS12-381 scalar field)
    signal input slot_number[64];   //F_p (BLS12-381 scalar field)
    signal input note_nonce[256];
    signal input nullifier_secret_key[256];

    signal output r;    //The lottery ticket in F_p

    component hash = Sha256(864);

            //The b"lead" Tag in bits with big endian order
    hash.in[0] <== 0;
    hash.in[1] <== 1;
    hash.in[2] <== 1;
    hash.in[3] <== 0;
    hash.in[4] <== 1;
    hash.in[5] <== 1;
    hash.in[6] <== 0;
    hash.in[7] <== 0;
    hash.in[8] <== 0;
    hash.in[9] <== 1;
    hash.in[10] <== 1;
    hash.in[11] <== 0;
    hash.in[12] <== 0;
    hash.in[13] <== 1;
    hash.in[14] <== 0;
    hash.in[15] <== 1;
    hash.in[16] <== 0;
    hash.in[17] <== 1;
    hash.in[18] <== 1;
    hash.in[19] <== 0;
    hash.in[20] <== 0;
    hash.in[21] <== 0;
    hash.in[22] <== 0;
    hash.in[23] <== 1;
    hash.in[24] <== 0;
    hash.in[25] <== 1;
    hash.in[26] <== 1;
    hash.in[27] <== 0;
    hash.in[28] <== 0;
    hash.in[29] <== 1;
    hash.in[30] <== 0;
    hash.in[31] <== 0;

    for(var i=0; i<256; i++){
        hash.in[32+i] <== epoch_nonce[i];
        if(i<64){
            hash.in[288+i] <== slot_number[i];
        }
        hash.in[352+i] <== nullifier_secret_key[i];
        hash.in[608+i] <== note_nonce[i];
    }

    component intifier = Bits2Num(253);     //Because if the scalar field is 255 bits, we support every number of 254 bits (not all of 255) and we can only compare numbers of 253 bits since we need 1 bit for sign.

    for(var i=0; i<253; i++){
        intifier.in[i] <== hash.out[253-i];
    }

    r <== intifier.out;
}

template check_lottery(){
    signal input pre_computed_threshold;    //253 bits
    signal input v;                         //253 bits max
    signal input epoch_nonce[256];
    signal input slot_number[64];
    signal input nullifier_secret_key[256];
    signal input note_nonce[256];

    component ticket = lottery_ticket();

    for(var i=0; i<256; i++){
        ticket.epoch_nonce[i] <== epoch_nonce[i];
        if(i<64){
            ticket.slot_number[i] <== slot_number[i];
        }
        ticket.nullifier_secret_key[i] <== nullifier_secret_key[i];
        ticket.note_nonce[i] <== note_nonce[i];
    }

    component isLess = BLSLessThan(253);

    isLess.in[0] <== ticket.r;
    isLess.in[1] <== pre_computed_threshold * v;
    //isLess.out === 1;
}

template nullifier_computer(){
    signal input note_nonce[256];
    signal input nullifier_public_key[256];
    signal input v[256];
    signal output nullifier[256];

    component hash = Sha256(880);

            //The b"coin-nullifier" Tag in bits with big endian order
    hash.in[0] <== 0;
    hash.in[1] <== 1;
    hash.in[2] <== 1;
    hash.in[3] <== 0;
    hash.in[4] <== 0;
    hash.in[5] <== 0;
    hash.in[6] <== 1;
    hash.in[7] <== 1;
    hash.in[8] <== 0;
    hash.in[9] <== 1;
    hash.in[10] <== 1;
    hash.in[11] <== 0;
    hash.in[12] <== 1;
    hash.in[13] <== 1;
    hash.in[14] <== 1;
    hash.in[15] <== 1;
    hash.in[16] <== 0;
    hash.in[17] <== 1;
    hash.in[18] <== 1;
    hash.in[19] <== 0;
    hash.in[20] <== 1;
    hash.in[21] <== 0;
    hash.in[22] <== 0;
    hash.in[23] <== 1;
    hash.in[24] <== 0;
    hash.in[25] <== 1;
    hash.in[26] <== 1;
    hash.in[27] <== 0;
    hash.in[28] <== 1;
    hash.in[29] <== 1;
    hash.in[30] <== 1;
    hash.in[31] <== 0;
    hash.in[32] <== 0;
    hash.in[33] <== 0;
    hash.in[34] <== 1;
    hash.in[35] <== 0;
    hash.in[36] <== 1;
    hash.in[37] <== 1;
    hash.in[38] <== 0;
    hash.in[39] <== 1;
    hash.in[40] <== 0;
    hash.in[41] <== 1;
    hash.in[42] <== 1;
    hash.in[43] <== 0;
    hash.in[44] <== 1;
    hash.in[45] <== 1;
    hash.in[46] <== 1;
    hash.in[47] <== 0;
    hash.in[48] <== 0;
    hash.in[49] <== 1;
    hash.in[50] <== 1;
    hash.in[51] <== 1;
    hash.in[52] <== 0;
    hash.in[53] <== 1;
    hash.in[54] <== 0;
    hash.in[55] <== 1;
    hash.in[56] <== 0;
    hash.in[57] <== 1;
    hash.in[58] <== 1;
    hash.in[59] <== 0;
    hash.in[60] <== 1;
    hash.in[61] <== 1;
    hash.in[62] <== 0;
    hash.in[63] <== 0;
    hash.in[64] <== 0;
    hash.in[65] <== 1;
    hash.in[66] <== 1;
    hash.in[67] <== 0;
    hash.in[68] <== 1;
    hash.in[69] <== 1;
    hash.in[70] <== 0;
    hash.in[71] <== 0;
    hash.in[72] <== 0;
    hash.in[73] <== 1;
    hash.in[74] <== 1;
    hash.in[75] <== 0;
    hash.in[76] <== 1;
    hash.in[77] <== 0;
    hash.in[78] <== 0;
    hash.in[79] <== 1;
    hash.in[80] <== 0;
    hash.in[81] <== 1;
    hash.in[82] <== 1;
    hash.in[83] <== 0;
    hash.in[84] <== 0;
    hash.in[85] <== 1;
    hash.in[86] <== 1;
    hash.in[87] <== 0;
    hash.in[88] <== 0;
    hash.in[89] <== 1;
    hash.in[90] <== 1;
    hash.in[91] <== 0;
    hash.in[92] <== 1;
    hash.in[93] <== 0;
    hash.in[94] <== 0;
    hash.in[95] <== 1;
    hash.in[96] <== 0;
    hash.in[97] <== 1;
    hash.in[98] <== 1;
    hash.in[99] <== 0;
    hash.in[100] <== 0;
    hash.in[101] <== 1;
    hash.in[102] <== 0;
    hash.in[103] <== 1;
    hash.in[104] <== 0;
    hash.in[105] <== 1;
    hash.in[106] <== 1;
    hash.in[107] <== 1;
    hash.in[108] <== 0;
    hash.in[109] <== 0;
    hash.in[110] <== 1;
    hash.in[111] <== 0;

    for(var i=0; i<256; i++){
        hash.in[112+i] <== note_nonce[i];
        hash.in[368+i] <== nullifier_public_key[i];
        hash.in[624+i] <== v[i];
    }
    for(var i=0; i<256; i++){
        nullifier[i] <== hash.out[i];
    }
}

template commitment_computer(){     // TODO: ensure all field are hash
    signal input note_nonce[256];
    signal input nullifier_public_key[256];
    signal input v[256];
    signal output commitment[256];

    component hash = Sha256(888);

            //The b"coin-commitment" Tag in bits with big endian order
    hash.in[0] <== 0;
    hash.in[1] <== 1;
    hash.in[2] <== 1;
    hash.in[3] <== 0;
    hash.in[4] <== 0;
    hash.in[5] <== 0;
    hash.in[6] <== 1;
    hash.in[7] <== 1;
    hash.in[8] <== 0;
    hash.in[9] <== 1;
    hash.in[10] <== 1;
    hash.in[11] <== 0;
    hash.in[12] <== 1;
    hash.in[13] <== 1;
    hash.in[14] <== 1;
    hash.in[15] <== 1;
    hash.in[16] <== 0;
    hash.in[17] <== 1;
    hash.in[18] <== 1;
    hash.in[19] <== 0;
    hash.in[20] <== 1;
    hash.in[21] <== 0;
    hash.in[22] <== 0;
    hash.in[23] <== 1;
    hash.in[24] <== 0;
    hash.in[25] <== 1;
    hash.in[26] <== 1;
    hash.in[27] <== 0;
    hash.in[28] <== 1;
    hash.in[29] <== 1;
    hash.in[30] <== 1;
    hash.in[31] <== 0;
    hash.in[32] <== 0;
    hash.in[33] <== 0;
    hash.in[34] <== 1;
    hash.in[35] <== 0;
    hash.in[36] <== 1;
    hash.in[37] <== 1;
    hash.in[38] <== 0;
    hash.in[39] <== 1;
    hash.in[40] <== 0;
    hash.in[41] <== 1;
    hash.in[42] <== 1;
    hash.in[43] <== 0;
    hash.in[44] <== 0;
    hash.in[45] <== 0;
    hash.in[46] <== 1;
    hash.in[47] <== 1;
    hash.in[48] <== 0;
    hash.in[49] <== 1;
    hash.in[50] <== 1;
    hash.in[51] <== 0;
    hash.in[52] <== 1;
    hash.in[53] <== 1;
    hash.in[54] <== 1;
    hash.in[55] <== 1;
    hash.in[56] <== 0;
    hash.in[57] <== 1;
    hash.in[58] <== 1;
    hash.in[59] <== 0;
    hash.in[60] <== 1;
    hash.in[61] <== 1;
    hash.in[62] <== 0;
    hash.in[63] <== 1;
    hash.in[64] <== 0;
    hash.in[65] <== 1;
    hash.in[66] <== 1;
    hash.in[67] <== 0;
    hash.in[68] <== 1;
    hash.in[69] <== 1;
    hash.in[70] <== 0;
    hash.in[71] <== 1;
    hash.in[72] <== 0;
    hash.in[73] <== 1;
    hash.in[74] <== 1;
    hash.in[75] <== 0;
    hash.in[76] <== 1;
    hash.in[77] <== 0;
    hash.in[78] <== 0;
    hash.in[79] <== 1;
    hash.in[80] <== 0;
    hash.in[81] <== 1;
    hash.in[82] <== 1;
    hash.in[83] <== 1;
    hash.in[84] <== 0;
    hash.in[85] <== 1;
    hash.in[86] <== 0;
    hash.in[87] <== 0;
    hash.in[88] <== 0;
    hash.in[89] <== 1;
    hash.in[90] <== 1;
    hash.in[91] <== 0;
    hash.in[92] <== 1;
    hash.in[93] <== 1;
    hash.in[94] <== 0;
    hash.in[95] <== 1;
    hash.in[96] <== 0;
    hash.in[97] <== 1;
    hash.in[98] <== 1;
    hash.in[99] <== 0;
    hash.in[100] <== 0;
    hash.in[101] <== 1;
    hash.in[102] <== 0;
    hash.in[103] <== 1;
    hash.in[104] <== 0;
    hash.in[105] <== 1;
    hash.in[106] <== 1;
    hash.in[107] <== 0;
    hash.in[108] <== 1;
    hash.in[109] <== 1;
    hash.in[110] <== 1;
    hash.in[111] <== 0;
    hash.in[112] <== 0;
    hash.in[113] <== 1;
    hash.in[114] <== 1;
    hash.in[115] <== 1;
    hash.in[116] <== 0;
    hash.in[117] <== 1;
    hash.in[118] <== 0;
    hash.in[119] <== 0;

    for(var i=0; i<256; i++){
        hash.in[120+i] <== note_nonce[i];
        hash.in[376+i] <== nullifier_public_key[i];
        hash.in[632+i] <== v[i];
    }
    for(var i=0; i<256; i++){
        commitment[i] <== hash.out[i];
    }
}

template nonce_updater(){
    signal input note_nonce[256];
    signal input nullifier_secret_key[256];
    signal output updated_nonce[256];

    component hash = Sha256(600);

            //The b"coin-evolve" Tag in bits with big endian order
    hash.in[0] <== 0;
    hash.in[1] <== 1;
    hash.in[2] <== 1;
    hash.in[3] <== 0;
    hash.in[4] <== 0;
    hash.in[5] <== 0;
    hash.in[6] <== 1;
    hash.in[7] <== 1;
    hash.in[8] <== 0;
    hash.in[9] <== 1;
    hash.in[10] <== 1;
    hash.in[11] <== 0;
    hash.in[12] <== 1;
    hash.in[13] <== 1;
    hash.in[14] <== 1;
    hash.in[15] <== 1;
    hash.in[16] <== 0;
    hash.in[17] <== 1;
    hash.in[18] <== 1;
    hash.in[19] <== 0;
    hash.in[20] <== 1;
    hash.in[21] <== 0;
    hash.in[22] <== 0;
    hash.in[23] <== 1;
    hash.in[24] <== 0;
    hash.in[25] <== 1;
    hash.in[26] <== 1;
    hash.in[27] <== 0;
    hash.in[28] <== 1;
    hash.in[29] <== 1;
    hash.in[30] <== 1;
    hash.in[31] <== 0;
    hash.in[32] <== 0;
    hash.in[33] <== 0;
    hash.in[34] <== 1;
    hash.in[35] <== 0;
    hash.in[36] <== 1;
    hash.in[37] <== 1;
    hash.in[38] <== 0;
    hash.in[39] <== 1;
    hash.in[40] <== 0;
    hash.in[41] <== 1;
    hash.in[42] <== 1;
    hash.in[43] <== 0;
    hash.in[44] <== 0;
    hash.in[45] <== 1;
    hash.in[46] <== 0;
    hash.in[47] <== 1;
    hash.in[48] <== 0;
    hash.in[49] <== 1;
    hash.in[50] <== 1;
    hash.in[51] <== 1;
    hash.in[52] <== 0;
    hash.in[53] <== 1;
    hash.in[54] <== 1;
    hash.in[55] <== 0;
    hash.in[56] <== 0;
    hash.in[57] <== 1;
    hash.in[58] <== 1;
    hash.in[59] <== 0;
    hash.in[60] <== 1;
    hash.in[61] <== 1;
    hash.in[62] <== 1;
    hash.in[63] <== 1;
    hash.in[64] <== 0;
    hash.in[65] <== 1;
    hash.in[66] <== 1;
    hash.in[67] <== 0;
    hash.in[68] <== 1;
    hash.in[69] <== 1;
    hash.in[70] <== 0;
    hash.in[71] <== 0;
    hash.in[72] <== 0;
    hash.in[73] <== 1;
    hash.in[74] <== 1;
    hash.in[75] <== 1;
    hash.in[76] <== 0;
    hash.in[77] <== 1;
    hash.in[78] <== 1;
    hash.in[79] <== 0;
    hash.in[80] <== 0;
    hash.in[81] <== 1;
    hash.in[82] <== 1;
    hash.in[83] <== 0;
    hash.in[84] <== 0;
    hash.in[85] <== 1;
    hash.in[86] <== 0;
    hash.in[87] <== 1;

    for(var i=0; i<256; i++){
        hash.in[88+i] <== note_nonce[i];
        hash.in[344+i] <== nullifier_secret_key[i];
    }
    for(var i=0; i<256; i++){
        updated_nonce[i] <== hash.out[i];
    }
}

template sha_2_to_1(){
    signal input a[256];
    signal input b[256];
    signal output hash_ab[256];

    component hash = Sha256(512);
    for(var i=0; i<256; i++){
        hash.in[i] <== a[i];
        hash.in[i+256] <== b[i];
    }

    for(var i=0;i<256;i++){
        hash_ab[i] <== hash.out[i];
    }
}

template membership_checker(){
    signal input commitment[256];           //The note commitment
    signal input commitments_root[256];     //The root of the Merkle Tree containing every commitments (of depth 32)
    signal input index[32];                 //Position of the note commitment in bits in big endian
    signal input node[32][256];            //Complementary hashes

    component hash[32];

    for(var i=0; i<32; i++){
        hash[i] = sha_2_to_1();
    }

    


    for(var i=0; i<256; i++){
        hash[0].a[i] <== commitment[i] - index[31] * (commitment[i] - node[0][i]);
        hash[0].b[i] <== node[0][i] - index[31] * (node[0][i] - commitment[i]);
    }

    for(var i=1; i<32; i++){
        for(var j=0; j<256; j++){
            hash[i].a[j] <== hash[i-1].hash_ab[j] - index[31] * (hash[i-1].hash_ab[j] - node[i][j]);
            hash[i].b[j] <== node[i][j] - index[31] * (node[i][j] - hash[i-1].hash_ab[j]);
        }
    }

    for(var i=0; i<256; i++){
        //commitments_root[i] === hash[31].hash_ab[i];
    }
}

template bits2num_253(){
    signal input bits[253];
    signal output value;

    signal intermediate_value[252];
    intermediate_value[0] <== bits[0] * 2;
    for(var i=1; i<252; i++){
        intermediate_value[i] <== intermediate_value[i-1] * 2 + bits[i];
    }
    value <== intermediate_value[251] * 2 + bits[252];
}

template check_bits(n){
    signal input bits[n];
    for(var i=0; i<n; i++){
        bits[i] * (1-bits[i]) === 0;
    }
}

template sha_proof_of_leadership(){
    signal input epoch_nonce[256];          //256 bits
    signal input slot_number[64];           //64 bits
    signal input pre_computed_threshold;    //253 bits
    signal input commitments_root[256];     //The root of the Merkle Tree containing every commitments (of depth 32) 256 bits

    signal input note_nonce[256];
    signal input nullifier_secret_key[256];
    signal input v[253];                    //253 bits
    signal input index[32];                 //Position of the note commitment in bits in big endian
    signal input node[32][256];             //Complementary hashes

    signal output nullifier[256];
    signal output updated_commiment[256];


        // Check that private inputs are indeed bits
    component bit_checker[36];
    for(var i=0; i<34; i++){
        bit_checker[i] = check_bits(256);
        if(i<32){
            for(var j=0; j<256; j++){
                bit_checker[i].bits[j] <== node[i][j];
            }
        }
    }
    bit_checker[34] = check_bits(253);
    bit_checker[35] = check_bits(32);
    for(var i=0; i<256; i++){
        bit_checker[32].bits[i] <== note_nonce[i];
        bit_checker[33].bits[i] <== nullifier_secret_key[i];
        if(i<253){
            bit_checker[34].bits[i] <== v[i];
        }
        if(i<32){
            bit_checker[35].bits[i] <== index[i];
        }
    }

        // Compute the value of v
    component bits2num = bits2num_253();
    for(var i=0; i<253; i++){
        bits2num.bits[i] <== v[i];
    }



        // Check that r < threshold
    component lottery_checker = check_lottery();
    lottery_checker.pre_computed_threshold <== pre_computed_threshold;
    lottery_checker.v <== bits2num.value;
    for(var i=0; i<256; i++){
        lottery_checker.epoch_nonce[i] <== epoch_nonce[i];
        if(i<64){
            lottery_checker.slot_number[i] <== slot_number[i];
        }
        lottery_checker.nullifier_secret_key[i] <== nullifier_secret_key[i];
        lottery_checker.note_nonce[i] <== note_nonce[i];
    }


        // Compute the note commitment
    component note_committer = commitment_computer();
    for(var i=0; i<256; i++){
        note_committer.note_nonce[i] <== note_nonce[i];
        note_committer.nullifier_public_key[i] <== nullifier_secret_key[i];      // TODO: reflect the nullifier public key computation later when defined
    }
    note_committer.v[0] <== 0;
    note_committer.v[1] <== 0;
    note_committer.v[2] <== 0;
    for(var i=0; i<253; i++){
        note_committer.v[i+3] <== v[i];
    }

        // Check the commitment membership
    component membership_checker = membership_checker();
    for(var i=0; i<256; i++){
        membership_checker.commitment[i] <== note_committer.commitment[i];
        membership_checker.commitments_root[i] <== commitments_root[i];
        for(var j=0; j<32; j++){
            if(i==0){
                membership_checker.index[j] <== index[j];
            }
            membership_checker.node[j][i] <== node[j][i];
        }
    }

        // Compute the note nullifier
    component nullifier_computer = nullifier_computer();
    for(var i=0; i<256; i++){
        nullifier_computer.note_nonce[i] <== note_nonce[i];
        nullifier_computer.nullifier_public_key[i] <== nullifier_secret_key[i]; // TODO: reflect the nullifier public key computation later when defined
    }
    nullifier_computer.v[0] <== 0;
    nullifier_computer.v[1] <== 0;
    nullifier_computer.v[2] <== 0;
    for(var i=0; i<253; i++){
        nullifier_computer.v[i+3] <== v[i];
    }
    for(var i=0; i<256; i++){
        nullifier[i] <== nullifier_computer.nullifier[i];
    }


        // Compute the evolved nonce
    component nonce_updater = nonce_updater();
    for(var i=0; i<256; i++){
        nonce_updater.note_nonce[i] <== note_nonce[i];
        nonce_updater.nullifier_secret_key[i] <== nullifier_secret_key[i];
    }

        // Compute the new note commitment
    component updated_note_committer = commitment_computer();
    for(var i=0; i<256; i++){
        updated_note_committer.note_nonce[i] <== nonce_updater.updated_nonce[i];
        updated_note_committer.nullifier_public_key[i] <== nullifier_secret_key[i];      // TODO: reflect the nullifier public key computation later when defined
    }
    updated_note_committer.v[0] <== 0;
    updated_note_committer.v[1] <== 0;
    updated_note_committer.v[2] <== 0;
    for(var i=0; i<253; i++){
        updated_note_committer.v[i+3] <== v[i];
    }
    for(var i =0; i<256; i++){
        updated_commiment[i] <== updated_note_committer.commitment[i];
    }

    
}


component main {public [epoch_nonce, slot_number, pre_computed_threshold, commitments_root]} = sha_proof_of_leadership();