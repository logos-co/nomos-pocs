//test
pragma circom 2.1.9;

include "../ledger/notes.circom";
include "../misc/constants.circom";

template proof_of_ownership(){
    signal input state;
    signal input value;
    signal input nonce;
    signal input zoneID;
    signal input secret_key;

    signal input attached_data;

    signal output commitment;

    component pk = derive_public_key();
    pk.secret_key <== secret_key;

    component cm = commitment();
    cm.state <== state;
    cm.value <== value;
    component nmo = NMO();
    cm.unit <== nmo.out;
    cm.nonce <== nonce;
    cm.zoneID <== zoneID;
    cm.public_key <== pk.out;

    // dummy constraint to avoid unused public input to be erased after compilation optimisation
    signal dummy;
    dummy <== attached_data * attached_data;
}

component main {public [zoneID,attached_data]}= proof_of_ownership();