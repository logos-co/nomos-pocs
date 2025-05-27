//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../ledger/notes.circom";
include "../ledger/merkle.circom";
include "../misc/comparator.circom";
include "../circomlib/circuits/bitify.circom";
include "../misc/constants.circom";


template ticket_calculator(){
    signal input epoch_nonce;
    signal input slot;
    signal input note_id;
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(5);
    component dst = LEAD_V1();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== epoch_nonce;
    hash.inp[2] <== slot;
    hash.inp[3] <== note_id;
    hash.inp[4] <== secret_key;

    out <== hash.out;
}

template derive_secret_key(){
    signal input starting_slot;
    signal input secrets_root;
    signal output out;

    component hash = Poseidon2_hash(3);
    component dst = NOMOS_POL_SK_V1();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== starting_slot;
    hash.inp[2] <== secrets_root;

    out <== hash.out;
}

template derive_entropy(){
    signal input slot;
    signal input note_id;
    signal input secret_key;
    signal output out;

    component hash = Poseidon2_hash(4);
    component dst = NOMOS_NONCE_CONTRIB_V1();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== slot;
    hash.inp[2] <== note_id;
    hash.inp[3] <== secret_key;

    out <== hash.out;
}

template is_winning_leadership(secret_depth){
    signal input slot;
    signal input epoch_nonce;
    signal input t0;
    signal input t1;
    signal input slot_secret;
    signal input slot_secret_path[secret_depth];

    //Part of the note id proof of membership to prove aged
    signal input aged_nodes[32];
    signal input aged_selectors[32];         // must be bits
    signal input aged_root;

    //Used to derive the note identifier
    signal input transaction_hash;
    signal input output_number;
    
    //Part of the note id proof of membership to prove it's unspent
    signal input latest_nodes[32];
    signal input latest_selectors[32];         // must be bits
    signal input latest_root;

    //Part of the secret key
    signal input starting_slot;
    signal input secrets_root;

    // The winning note value
    signal input value;

    signal output out;
    signal output note_identifier;
    signal output secret_key;


    // Derive the secret key
    component sk = derive_secret_key();
    sk.starting_slot <== starting_slot;
    sk.secrets_root <== secrets_root;


    // Derive the public key from the secret key
    component pk = derive_public_key();
    pk.secret_key <== sk.out;


    // Derive the note id
    component note_id = Poseidon2_hash(5);
    component dst_note_id = NOMOS_NOTE_ID_V1();
    note_id.inp[0] <== dst_note_id.out;
    note_id.inp[1] <== transaction_hash;
    note_id.inp[2] <== output_number;
    note_id.inp[3] <== value;
    note_id.inp[4] <== pk.out;


    // Check the note ID is aged enough
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        aged_selectors[i] * (1 - aged_selectors[i]) === 0;
    }
            //Then check the proof of membership
    component aged_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        aged_membership.nodes[i] <== aged_nodes[i];
        aged_membership.selector[i] <== aged_selectors[i];
    }
    aged_membership.root <== aged_root;
    aged_membership.leaf <== note_id.out;


    // Compute the lottery ticket
    component ticket = ticket_calculator();
    ticket.epoch_nonce <== epoch_nonce;
    ticket.slot <== slot;
    ticket.note_id <== note_id.out;
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


    // Check that the note is unspent
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        latest_selectors[i] * (1 - latest_selectors[i]) === 0;
    }
            //Then check the note id is in the latest ledger state
    component unspent_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        unspent_membership.nodes[i] <== latest_nodes[i];
        unspent_membership.selector[i] <== latest_selectors[i];
    }
    unspent_membership.root <== latest_root;
    unspent_membership.leaf <== note_id.out;


    // Check the knowledge of the secret at position slot - starting_slot
            // Verify that the substraction wont underflow (starting_slot < slot)
    component checker = SafeLessEqThan(252);
    checker.in[0] <== starting_slot;
    checker.in[1] <== slot;
            // Compute the positions related to slot - starting_slot (and make sure it's 25 bits)
    component bits = Num2Bits(secret_depth);
    bits.in <== slot - starting_slot;
            // Check the membership of the secret_slot against the secrets_root
    component secret_membership = proof_of_membership(secret_depth);
    for(var i =0; i<secret_depth; i++){
        secret_membership.nodes[i] <== slot_secret_path[i];
        secret_membership.selector[i] <== bits.out[secret_depth-1-i];
    }
    secret_membership.root <== secrets_root;
    secret_membership.leaf <== slot_secret;

    // Check that every constraint holds
    signal intermediate_out[3];
    intermediate_out[0] <== aged_membership.out * winning.out;
    intermediate_out[1] <== unspent_membership.out * secret_membership.out;
    intermediate_out[2] <== intermediate_out[0] * intermediate_out[1];
    out <==  intermediate_out[2] * checker.out;

    note_identifier <== note_id.out;
    secret_key <== sk.out;
} 


template proof_of_leadership(secret_depth){
    signal input slot;
    signal input epoch_nonce;
    signal input t0;
    signal input t1;
    signal input slot_secret;
    signal input slot_secret_path[secret_depth];

    //Part of the note id proof of membership to prove aged
    signal input aged_nodes[32];
    signal input aged_selectors[32];         // must be bits
    signal input aged_root;

    //Used to derive the note identifier
    signal input transaction_hash;
    signal input output_number;
    
    //Part of the note id proof of membership to prove it's unspent
    signal input latest_nodes[32];
    signal input latest_selectors[32];         // must be bits
    signal input latest_root;

    //Part of the secret key
    signal input starting_slot;
    signal input secrets_root;

    // The winning note. The unit is supposed to be NMO and the ZoneID is MANTLE
    signal input value;

    // Verify the note is winning the lottery
    component lottery_checker = is_winning_leadership(secret_depth);
    lottery_checker.slot <== slot;
    lottery_checker.epoch_nonce <== epoch_nonce;
    lottery_checker.t0 <== t0;
    lottery_checker.t1 <== t1;
    lottery_checker.slot_secret <== slot_secret;
    for(var i = 0; i < secret_depth; i++){
        lottery_checker.slot_secret_path[i] <== slot_secret_path[i];
    }
    for(var i = 0; i < 32; i++){
        lottery_checker.aged_nodes[i] <== aged_nodes[i];
        lottery_checker.aged_selectors[i] <== aged_selectors[i];
        lottery_checker.latest_nodes[i] <== latest_nodes[i];
        lottery_checker.latest_selectors[i] <== latest_selectors[i];
    }
    lottery_checker.aged_root <== aged_root;
    lottery_checker.transaction_hash <== transaction_hash;
    lottery_checker.output_number <== output_number;
    lottery_checker.latest_root <== latest_root;
    lottery_checker.starting_slot <== starting_slot;
    lottery_checker.secrets_root <== secrets_root;
    lottery_checker.value <== value;

    lottery_checker.out === 1;


    // One time signing key used to sign the block proposal and the block
    signal input one_time_key;

    //Avoid the circom optimisation that removes unused public input
    signal dummy;
    dummy <== one_time_key * one_time_key;

    signal output entropy_contrib;


    // Compute the entropy contribution
    component entropy = derive_entropy();
    entropy.slot <== slot;
    entropy.note_id <== lottery_checker.note_identifier;
    entropy.secret_key <== lottery_checker.secret_key;

    entropy_contrib <== entropy.out;
}


component main {public [slot,epoch_nonce,t0,t1,aged_root,latest_root,one_time_key]}= proof_of_leadership(25);