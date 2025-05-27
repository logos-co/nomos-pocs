// PoQ.circom
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../misc/constants.circom";         // defines NOMOS_KDF, SELECTION_RANDOMNESS, PROOF_NULLIFIER
include "../misc/comparator.circom";        
include "../circomlib/circuits/bitify.circom";
include "../ledger/notes.circom";           // defines proof_of_membership
include "../Mantle/pol.circom";      // defines proof_of_leadership

/**
 * ProofOfQuota(nLevelsPK, nLevelsPol)
 *
 * - nLevelsPK   : depth of the core-node public-key registry Merkle tree
 * - nLevelsPol  : depth of the slot-secret tree used in PoL (25)
 * - bitsQuota   : bit-width for the index comparator
 */
template ProofOfQuota(nLevelsPK, nLevelsPol, bitsQuota) {
    // Public Inputs
    signal input session;       // session s
    signal input Qc;            // core quota Q_C
    signal input Ql;            // leadership quota Q_L
    signal input pk_root;       // Merkle root of registered core-node public keys
    signal input aged_root;     // PoL: aged notes root
    signal input latest_root;   // PoL: latest notes root
    signal input K;  // Blend: one-time signature public key

    signal output nullifier;    //key_nullifier

    // Private Inputs
    signal input selector;      // 0 = core, 1 = leader
    signal input index;         // nullifier index

    // Core-nodes inputs
    signal input core_sk;                       // core node secret key
    signal input core_path[nLevelsPK];          // Merkle path for core PK
    signal input core_selectors[nLevelsPK];     // path selectors (bits)

    // PoL branch inputs (all the PoL private data)
    signal input slot;
    signal input epoch_nonce;
    signal input t0;
    signal input t1;
    signal input slot_secret;
    signal input slot_secret_path[nLevelsPol];

    signal input aged_nodes[32];
    signal input aged_selectors[32];
    signal input transaction_hash;
    signal input output_number;
    signal input latest_nodes[32];
    signal input latest_selectors[32];



    // Constraints
    selector * (1 - selector) === 0;

    // derive pk_core = Poseidon(NOMOS_KDF || core_sk)
    component kdf = Poseidon2_hash(2);
    component dstKdf = NOMOS_KDF();
    kdf.inp[0] <== dstKdf.out;
    kdf.inp[1] <== core_sk;
    signal pk_core;
    pk_core <== kdf.out;

    // Merkle‐verify pk_core in pk_root
    component coreReg = proof_of_membership(nLevelsPK);
    for (var i = 0; i < nLevelsPK; i++) {
        core_selectors[i] * (1 - core_selectors[i]) === 0;
        coreReg.nodes[i]    <== core_path[i];
        coreReg.selector[i] <== core_selectors[i];
    }
    coreReg.root <== pk_root;
    coreReg.leaf <== pk_core;

    // enforce PoL
    // (All constraints inside pol ensure LeadershipVerify)
    // /!\ copy the PoL constraints here /!\
    component win = is_winning_leadership(nLevelsPol);
    win.slot                <== slot;
    win.epoch_nonce         <== epoch_nonce;
    win.t0                  <== t0;
    win.t1                  <== t1;
    win.slot_secret         <== slot_secret;
    for (var i = 0; i < nLevelsPol; i++) {
        win.slot_secret_path[i] <== slot_secret_path[i];
    }
    for (var i = 0; i < 32; i++) {
        win.aged_nodes[i]      <== aged_nodes[i];
        win.aged_selectors[i]  <== aged_selectors[i];
        win.latest_nodes[i]    <== latest_nodes[i];
        win.latest_selectors[i]<== latest_selectors[i];
    }
    win.aged_root      <== aged_root;
    win.transaction_hash <== transaction_hash;
    win.output_number    <== output_number;
    win.latest_root      <== latest_root;
    win.starting_slot  <== starting_slot;
    win.secrets_root   <== secrets_root;
    win.value          <== value;

    signal is_leader = win.out;  // 1 if PoL passed

    // Enforce the selected role is correct
    selector * (is_leader - coreReg.out) + coreReg.out === 1;



    // Quota check: index < Qc if core, index < Ql if leader
    component cmp = SafeLessThan(bitsQuota);
    cmp.a <== index;
    cmp.b <== selector * (Ql - Qc) + Qc;
    cmp.out === 1;

    // Derive selection_randomness
    component randomness = Poseidon2_hash(4);
    component dstSel = SELECTION_RANDOMNESS();
    randomness.inp[0] <== dstSel.out;
    // choose core_sk or pol.secret_key:
    randomness.inp[1] <== selector * (pol.secret_key - core_sk ) + core_sk;
    randomness.inp[2] <== index;
    randomness.inp[3] <== session;

    // Derive proof_nullifier
    component nf = Poseidon2_hash(2);
    component dstNF = PROOF_NULLIFIER();
    nf.inp[0] <== dstNF.out;
    nf.inp[1] <== randomness.out;
    nullifier <== nf.out;
}

// Instantiate with chosen depths: 32 for core PK tree, 25 for PoL slot tree
component main { public [ session, Qc, Ql, pk_root, aged_root, latest_root, K ] }
    = ProofOfQuota(32, 25, 6);