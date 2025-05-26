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

    signal output nullifier;

    // Private Inputs
    signal input selector;      // 0 = core, 1 = leader
    signal input index;         // nullifier index

    // Core-nodes inputs
    signal input core_sk;                       // core node secret key
    signal input core_path[nLevelsPK];          // Merkle path for core PK
    signal input core_selectors[nLevelsPK];     // path selectors (bits)

    // Leaders inputs (all PoL inputs)
    component pol = proof_of_leadership();


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
    is_winning <== //0 or 1 for PoL

    // Enforce the selected role is correct
    selector * (is_winning - coreReg.out) + coreReg.out === 1;



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