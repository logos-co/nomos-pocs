//test
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../ledger/merkle.circom";
include "../misc/constants.circom";

template derive_voucher_nullifier(){
    signal input secret_voucher;
    signal output out;

    component hash = Poseidon2_hash(2);
    component dst = VOUCHER_NF();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== secret_voucher;

    out <== hash.out;
}

template derive_reward_voucher(){
    signal input secret_voucher;
    signal output out;

    component hash = Poseidon2_hash(2);
    component dst = REWARD_VOUCHER();
    hash.inp[0] <== dst.out;
    hash.inp[1] <== secret_voucher;

    out <== hash.out;
}

template proof_of_claim(){
    signal input secret_voucher;
    signal input merkle_nodes[32];
    signal input selectors[32];
    signal input attached_data;
    signal input voucher_root;

    signal output voucher_nullifier;

    //derive the reward voucher
    component reward_voucher = derive_reward_voucher();
    reward_voucher.secret_voucher <== secret_voucher;

    //verify reward voucher membership
    component reward_membership = proof_of_membership(32);
    for(var i = 0; i < 32; i++){
        reward_membership.nodes[i] <== merkle_nodes[i];
        reward_membership.selector[i] <== selectors[i];
    }
    reward_membership.root <== voucher_root;
    reward_membership.leaf <== reward_voucher.out;

    reward_membership.out === 1;


    //derive the reward nullifier
    component reward_nullifier = derive_voucher_nullifier();
    reward_nullifier.secret_voucher <== secret_voucher;
    voucher_nullifier <== reward_nullifier.out;

    

    // dummy constraint to avoid unused public input to be erased after compilation optimisation
    signal dummy;
    dummy <== attached_data * attached_data;
}

component main {public [voucher_root,attached_data]}= proof_of_claim();