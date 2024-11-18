use serde::{Deserialize, Serialize};

pub struct TransferPublic {
    pub nf: Nullifier,
    pub cm: NoteCommitment,
    pub cm_root: [u8; 32],
}

pub struct WithdrawPublic {
    pub nf: Nullifier,
    pub cm_root: [u8; 32],
    pub authorized_pks: [u8; 32]    ,
}

pub struct WithdrawPrivate {
    pub input: InputWitness,
    pub cm_path: Vec<merkle::PathNode>,
}

pub struct TransferPrivate {
    pub input: InputWitness,
    pub cm_path: Vec<merkle::PathNode>,
    pub output: OutputWitness,
    pub authorized_pks: Vec<NullifierCommitment>
}
