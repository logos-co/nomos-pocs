//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../ledger/notes.circom";
include "../misc/comparator.circom";
include "../circomlib/circuits/bitify.circom";


template ticket_calculator(){
    signal input epoch_nonce;
    signal input slot;
    signal input commitment;
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(5);
    // int.from_bytes(hashlib.sha256(b"LEAD").digest()[:-1], "little") = 137836078329650723736739065075984465408055658421620421917147974048265460598
    hash.inp[0] <== 137836078329650723736739065075984465408055658421620421917147974048265460598;
    hash.inp[1] <== epoch_nonce;
    hash.inp[2] <== slot;
    hash.inp[3] <== commitment;
    hash.inp[4] <== secret_key;

    out <== hash.out;
}

template derive_secret_key(){
    signal input starting_slot;
    signal input secrets_root;
    signal output out;

    component hash = Poseidon2_hash(3);
    // int.from_bytes(hashlib.sha256(b"NOMOS_SECRET_KEY").digest()[:-1], "little") = 344114695764831179145057610008294480248205750382057360672614582644594850870
    hash.inp[0] <== 344114695764831179145057610008294480248205750382057360672614582644594850870;
    hash.inp[1] <== starting_slot;
    hash.inp[2] <== secrets_root;

    out <== hash.out;
}

template derive_entropy(){
    signal input slot;
    signal input commitment;
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(4);
    // int.from_bytes(hashlib.sha256(b"NOMOS_NONCE_CONTRIB").digest()[:-1], "little") = 193275670388587576544090216996849534520361117581542778964162861667418671481
    hash.inp[0] <== 193275670388587576544090216996849534520361117581542778964162861667418671481;
    hash.inp[1] <== slot;
    hash.inp[2] <== commitment;
    hash.inp[3] <== secret_key;

    out <== hash.out;
}


template payment_proof_of_leadership(){
    signal input slot;
    signal input epoch_nonce;
    signal input t0;
    signal input t1;
    signal input slot_secret;
    signal input slot_secret_path[25];

    //Part of the commitment proof of membership
    signal input cm_nodes[32];
    signal input cm_selectors[32];         // must be bits
    signal input commitments_root;

    //Part of the nullifier proof of non-membership
    signal input nf_previous;
    signal input nf_next;
    signal input nf_nodes[32];
    signal input nf_selectors[32];
    signal input nullifiers_root;

    //Part of the secret key
    signal input starting_slot;
    signal input secrets_root;

    // The winning note. The unit is supposed to be NMO and the ZoneID is PAYMENT
    signal input state;
    signal input value;
    signal input nonce;

    signal input one_time_key;

    //Avoid the circom optimisation that removes unused public input
    signal dummy;
    dummy <== one_time_key * one_time_key;

    signal output entropy_contrib;


    // Derive the secret key
    component sk = derive_secret_key();
    sk.starting_slot <== starting_slot;
    sk.secrets_root <== secrets_root;


    // Derive the public key from the secret key
    component pk = derive_public_key();
    pk.secret_key <== sk.out;


    // Derive the commitment from the note and the public key
    component cm = commitment();
    cm.state <== state;
    cm.value <== value;
    // int.from_bytes(hashlib.sha256(b"NMO").digest()[:-1], "little") = 161796427070100155131822184769584603407573991022311108406630770340454367555
    cm.unit <== 161796427070100155131822184769584603407573991022311108406630770340454367555;
    cm.nonce <== nonce;
    // int.from_bytes(hashlib.sha256(b"PAYMENT").digest()[:-1], "little") = 281646683567839822174419720505039861445414630574005374635737888376398200354
    cm.zoneID <== 281646683567839822174419720505039861445414630574005374635737888376398200354;
    cm.public_key <== pk.out;


    // Derive the nullifier from the commitment and the secret key
    component nf = nullifier();
    nf.commitment <== cm.out;
    nf.secret_key <== sk.out;


    // Check commitment membership
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        cm_selectors[i] * (1 - cm_selectors[i]) === 0;
    }
            //Then check the proof of membership
    component cm_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        cm_membership.nodes[i] <== cm_nodes[i];
        cm_membership.selector[i] <== cm_selectors[i];
    }
    cm_membership.root <== commitments_root;
    cm_membership.leaf <== cm.out;


    // Compute the lottery ticket
    component ticket = ticket_calculator();
    ticket.epoch_nonce <== epoch_nonce;
    ticket.slot <== slot;
    ticket.commitment <== cm.out;
    ticket.secret_key <== sk.out;


    // Compute the lottery threshold
    signal intermediate;
    signal threshold;
    intermediate <== t1 * value;
    threshold <== value * (t0 + intermediate);


    // Check that the ticket is winning
    component winning = FullLessThan();
    winning.a <== ticket.out;
    winning.b <== threshold;
    winning.out === 1;


    // Check nullifier non-membership
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        nf_selectors[i] * (1 - nf_selectors[i]) === 0;
    }
            //Then check the proof of non-membership
    component nf_membership = proof_of_non_membership(32);
    nf_membership.previous <== nf_previous;
    nf_membership.nullifier <== nf.out;
    nf_membership.next <== nf_next;
    nf_membership.root <== nullifiers_root;
    for(var i =0; i<32; i++){
        nf_membership.nodes[i] <== nf_nodes[i];
        nf_membership.selector[i] <== nf_selectors[i];
    }


    // Check the knowledge of the secret at position slot - starting_slot
            // Verify that the substraction wont underflow (starting_slot < slot)
    component checker = SafeLessEqThan(252);
    checker.in[0] <== starting_slot;
    checker.in[1] <== slot;
    checker.out === 1;
            // Compute the positions related to slot - starting_slot
    component bits = Num2Bits(25);
    bits.in <== slot - starting_slot;
            // Check the membership of the secret_slot against the secrets_root
    component secret_membership = proof_of_membership(25);
    for(var i =0; i<25; i++){
        secret_membership.nodes[i] <== slot_secret_path[i];
        secret_membership.selector[i] <== bits.out[24-i];
    }
    secret_membership.root <== secrets_root;
    secret_membership.leaf <== slot_secret;


    // Compute the entropy contribution
    component entropy = derive_entropy();
    entropy.slot <== slot;
    entropy.commitment <== cm.out;
    entropy.secret_key <== sk.out;

    entropy_contrib <== entropy.out;
} 

component main {public [slot,epoch_nonce,t0,t1,commitments_root,nullifiers_root,one_time_key]}= payment_proof_of_leadership();