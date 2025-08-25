// PoQ.circom
pragma circom 2.1.9;

include "../hash_bn/poseidon2_hash.circom";
include "../misc/constants.circom";         // defines NOMOS_KDF, SELECTION_RANDOMNESS, PROOF_NULLIFIER
include "../misc/comparator.circom";        
include "../circomlib/circuits/bitify.circom";
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
    signal input core_quota;
    signal input leader_quota;
    signal input core_root;
    signal input pol_ledger_aged;     // PoL: aged notes root
    signal input K_part_one;  // Blend: one-time signature public key
    signal input K_part_two;  // Blend: one-time signature public key

    signal dummy_one;
    dummy_one <== K_part_one * K_part_one;
    signal dummy_two;
    dummy_two <== K_part_two * K_part_two;

    signal output key_nullifier;    //key_nullifier

    // Private Inputs
    signal input selector;      // 0 = core, 1 = leader
    signal input index;         // nullifier index

    // Core-nodes inputs
    signal input core_sk;                       // core node secret key
    signal input core_path[nLevelsPK];          // Merkle path for core PK
    signal input core_path_selectors[nLevelsPK];     // path selectors (bits)

    // PoL branch inputs (all the PoL private data)
    signal input pol_sl;
    signal input pol_epoch_nonce;
    signal input pol_t0;
    signal input pol_t1;
    signal input pol_slot_secret;
    signal input pol_slot_secret_path[nLevelsPol];

    signal input pol_noteid_path[32];
    signal input pol_noteid_path_selectors[32];
    signal input pol_note_tx_hash;
    signal input pol_note_output_number;

    signal input pol_sk_starting_slot;
    signal input secrets_root;    // THIS NEEDS TO BE REMOVED
    signal input pol_note_value;



    // Constraints
    selector * (1 - selector) === 0;

    // derive pk_core = Poseidon(NOMOS_KDF || core_sk)
    component kdf = Poseidon2_hash(2);
    component dstKdf = NOMOS_KDF_V1();
    kdf.inp[0] <== dstKdf.out;
    kdf.inp[1] <== core_sk;
    signal pk_core;
    pk_core <== kdf.out;

    // Merkle‐verify pk_core in core_root
    component coreReg = proof_of_membership(nLevelsPK);
    for (var i = 0; i < nLevelsPK; i++) {
        core_path_selectors[i] * (1 - core_path_selectors[i]) === 0;
        coreReg.nodes[i]    <== core_path[i];
        coreReg.selector[i] <== core_path_selectors[i];
    }
    coreReg.root <== core_root;
    coreReg.leaf <== pk_core;

    // enforce potential PoL (without verification that the note is unspent)
    // (All constraints inside pol ensure LeadershipVerify)
    component would_win = would_win_leadership(nLevelsPol);
    would_win.slot                <== pol_sl;
    would_win.epoch_nonce         <== pol_epoch_nonce;
    would_win.t0                  <== pol_t0;
    would_win.t1                  <== pol_t1;
    would_win.slot_secret         <== pol_slot_secret;
    for (var i = 0; i < nLevelsPol; i++) {
        would_win.slot_secret_path[i] <== pol_slot_secret_path[i];
    }
    for (var i = 0; i < 32; i++) {
        would_win.aged_nodes[i]      <== pol_noteid_path[i];
        would_win.aged_selectors[i]  <== pol_noteid_path_selectors[i];
    }
    would_win.aged_root      <== pol_ledger_aged;
    would_win.transaction_hash <== pol_note_tx_hash;
    would_win.output_number    <== pol_note_output_number;
    would_win.starting_slot  <== pol_sk_starting_slot;
    would_win.secrets_root   <== secrets_root;
    would_win.value          <== pol_note_value;

    // Enforce the selected role is correct
    selector * (would_win.out - coreReg.out) + coreReg.out === 1;




    // Quota check: index < core_quota if core, index < leader_quota if leader
    component cmp = SafeLessThan(bitsQuota);
    cmp.in[0] <== index;
    cmp.in[1] <== selector * (leader_quota - core_quota) + core_quota;
    cmp.out === 1;

    // Derive selection_randomness
    component randomness = Poseidon2_hash(4);
    component dstSel = SELECTION_RANDOMNESS_V1();
    randomness.inp[0] <== dstSel.out;
    // choose core_sk or pol.secret_key:
    randomness.inp[1] <== selector * (would_win.secret_key - core_sk ) + core_sk;
    randomness.inp[2] <== index;
    randomness.inp[3] <== session;

    // Derive key_nullifier
    component nf = Poseidon2_hash(2);
    component dstNF = PROOF_NULLIFIER_V1();         // THIS NEEDS TO BE UPDATED
    nf.inp[0] <== dstNF.out;
    nf.inp[1] <== randomness.out;
    key_nullifier <== nf.out;
}

// Instantiate with chosen depths: 20 for core PK tree, 25 for PoL secret slot tree
component main { public [ session, core_quota, leader_quota, core_root, pol_ledger_aged, K_part_one, K_part_two ] }
    = ProofOfQuota(20, 25, 20);