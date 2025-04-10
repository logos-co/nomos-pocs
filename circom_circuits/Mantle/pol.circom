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


template proof_of_leadership(){
    signal input selector;              // 0 if the note is shielded and 1 if the note is unshielded

    // Check that the selector is indeed a bit
    selector * (1- selector) === 0;

    signal input slot;
    signal input epoch_nonce;
    signal input t0;
    signal input t1;
    signal input slot_secret;
    signal input slot_secret_path[25];

    //Part of the commitment or note id proof of membership to prove aged
    signal input aged_nodes[32];
    signal input aged_selectors[32];         // must be bits
    signal input commitments_aged_root;
    signal input note_id_aged_root;


    //Used to derive the note identifier, it can be dumb inputs if it's a shielded note
    signal input transaction_hash;
    signal input output_number;
    
    
    //Part of the nullifer proof of non-membership/commitment proof of membership to prove the note is unspent
    signal input nf_previous;       // Can be mocked and set to any value if selector == 1 as long as previous < nullifier < next
    signal input nf_next;
    signal input unspent_nodes[32];
    signal input unspent_selectors[32];         // must be bits
    signal input nf_unspent_root;
    signal input note_id_unspent_root;

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


    // Derive the nullifier from the commitment and the secret key
    component nf = nullifier();
    nf.commitment <== cm.out;
    nf.secret_key <== sk.out;


    // Derive the note id
    component note_id = Poseidon2_hash(4);
    component dst_note_id = NOMOS_NOTE_ID();
    note_id.inp[0] <== dst_note_id.out;
    note_id.inp[1] <== transaction_hash;
    note_id.inp[2] <== output_number;
    note_id.inp[3] <== cm.out;

    // Check commitment membership (is aged enough)
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
    aged_membership.root <==  (note_id_aged_root - commitments_aged_root) * selector + commitments_aged_root;
    aged_membership.leaf <== (note_id.out - cm.out) * selector + cm.out;


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


    // Check that the note is unspent
            //First check selectors are indeed bits
    for(var i = 0; i < 32; i++){
        unspent_selectors[i] * (1 - unspent_selectors[i]) === 0;
    }
            //Then check the proof of membership (that the nullifier leaf is in the set or that the note identifier is)
    component unspent_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        unspent_membership.nodes[i] <== unspent_nodes[i];
        unspent_membership.selector[i] <== unspent_selectors[i];
    }
    unspent_membership.root <== (note_id_unspent_root - nf_unspent_root) * selector + nf_unspent_root;
            //Compute the leaf if it's a private note representing previous nf pointing to next in the IMT
    component hash = Poseidon2_hash(2);
    hash.inp[0] <== nf_previous; 
    hash.inp[1] <== nf_next;
    unspent_membership.leaf <== (note_id.out - hash.out) * selector + hash.out;  // the leaf is then either the note identifier or the leaf computed before

            // Check that nullifier stictly falls between previous and next if the note is private.
            // If the note is public previous and next can be any values such that previous < nullifier < next
    component comparator[2];
    comparator[0] = SafeFullLessThan();
    comparator[0].a <== nf_previous;
    comparator[0].b <== nf.out;
    comparator[0].out === 1;
    comparator[1] = SafeFullLessThan();
    comparator[1].a <== nf.out;
    comparator[1].b <== nf_next;
    comparator[1].out === 1;


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

component main {public [slot,epoch_nonce,t0,t1,commitments_aged_root,note_id_aged_root,nf_unspent_root,note_id_unspent_root,one_time_key]}= proof_of_leadership();