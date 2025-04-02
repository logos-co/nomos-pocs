//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../ledger/notes.circom";
include "../misc/comparator.circom";
include "../circomlib/circuits/bitify.circom";
include "../misc/constants.circom";


template ticket_calculator(){
    signal input epoch_nonce;
    signal input slot;
    signal input commitment;
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(5);
    component dst = LEAD();
    hash.inp[0] <== dst.out;
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
    component dst = NOMOS_POL_SK();
    hash.inp[0] <== dst.out;
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
    component dst = NOMOS_NONCE_CONTRIB();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== slot;
    hash.inp[2] <== commitment;
    hash.inp[3] <== secret_key;

    out <== hash.out;
}


template staking_proof_of_leadership(){
    signal input slot;
    signal input epoch_nonce;
    signal input t0;
    signal input t1;
    signal input slot_secret;
    signal input slot_secret_path[25];

    //Part of the commitment proof of membership to prove aged
    signal input cm_aged_nodes[32];
    signal input cm_aged_selectors[32];         // must be bits
    signal input commitments_aged_root;

    //Part of the commitment proof of membership to prove aged
    signal input cm_unspent_nodes[32];
    signal input cm_unspent_selectors[32];         // must be bits
    signal input commitments_unspent_root;

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
    component nmo = NMO();
    cm.unit <== nmo.out;
    cm.nonce <== nonce;
    component staking = STAKING();
    cm.zoneID <== staking.out;
    cm.public_key <== pk.out;


    // Check commitment membership (is aged enough)
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        cm_aged_selectors[i] * (1 - cm_aged_selectors[i]) === 0;
    }
            //Then check the proof of membership
    component cm_aged_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        cm_aged_membership.nodes[i] <== cm_aged_nodes[i];
        cm_aged_membership.selector[i] <== cm_aged_selectors[i];
    }
    cm_aged_membership.root <== commitments_aged_root;
    cm_aged_membership.leaf <== cm.out;


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


    // Check commitment membership (is unspent)
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        cm_unspent_selectors[i] * (1 - cm_unspent_selectors[i]) === 0;
    }
            //Then check the proof of membership
    component cm_unspent_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        cm_unspent_membership.nodes[i] <== cm_unspent_nodes[i];
        cm_unspent_membership.selector[i] <== cm_unspent_selectors[i];
    }
    cm_unspent_membership.root <== commitments_unspent_root;
    cm_unspent_membership.leaf <== cm.out;


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

component main {public [slot,epoch_nonce,t0,t1,commitments_aged_root,commitments_unspent_root,one_time_key]}= staking_proof_of_leadership();