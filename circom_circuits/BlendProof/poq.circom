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
template ProofOfQuota(nLevelsPK, nLevelsPol) {
    // Public Inputs
    signal input session;       // session s
    signal input Qc;            // core quota Q_C
    signal input Ql;            // leadership quota Q_L
    signal input pk_root;       // Merkle root of registered core-node public keys
    signal input aged_root;     // PoL: aged notes root
    signal input latest_root;   // PoL: latest notes root
    signal input one_time_key;  // Blend: one-time signature public key

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
    signal pk_core = kdf.out;

    // Merkle‐verify pk_core in pk_root
    component coreReg = proof_of_membership(nLevelsPK);
    for (var i = 0; i < nLevelsPK; i++) {
        coreReg.nodes[i]    <== core_path[i];
        coreReg.selector[i] <== core_selectors[i];
    }
    coreReg.root <== pk_root;
    coreReg.leaf <== pk_core;

    // enforce PoL
    // (All constraints inside pol ensure LeadershipVerify)

    // Quota check: index < Qc if core, index < Ql if leader
    component cmpC = LessThan();
    cmpC.a <== index; cmpC.b <== Qc;
    component cmpL = LessThan();
    cmpL.a <== index; cmpL.b <== Ql;

    // (1-selector)*cmpC.out + selector*cmpL.out == 1
    signal inQuota = (1 - selector) * cmpC.out + selector * cmpL.out;
    inQuota === 1;

    // Derive selection_randomness
    component mix = Poseidon2_hash(4);
    component dstSel = SELECTION_RANDOMNESS();
    mix.inp[0] <== dstSel.out;
    // choose core_sk or pol.secret_key:
    mix.inp[1] <== (1 - selector) * core_sk + selector * pol.secret_key;
    mix.inp[2] <== index;
    mix.inp[3] <== session;
    signal selRand = mix.out;

    // Derive proof_nullifier
    component nf = Poseidon2_hash(2);
    component dstNF = PROOF_NULLIFIER();
    nf.inp[0] <== dstNF.out;
    nf.inp[1] <== selRand;
    signal proofNullifier = nf.out;

    // Public outputs
    signal output K;
    signal output nullifier;
    K         <== one_time_key;
    nullifier <== proofNullifier;
}

// Instantiate with chosen depths: 32 for core PK tree, 25 for PoL slot tree
component main { public [ session, Qc, Ql, pk_root, aged_root, latest_root, one_time_key ] }
    = ProofOfQuota(32, 25);