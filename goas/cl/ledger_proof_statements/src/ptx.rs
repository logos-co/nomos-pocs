use cl::{merkle, InputWitness, OutputWitness};
use serde::{Deserialize, Serialize};

/// An input to a partial transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialTxInputPrivate {
    pub input: InputWitness,
    pub path: Vec<merkle::PathNode>,
}

impl PartialTxInputPrivate {
    pub fn input_root(&self) -> [u8; 32] {
        let leaf = merkle::leaf(&self.input.commit().to_bytes());
        merkle::path_root(leaf, &self.path)
    }
}

/// An output to a partial transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialTxOutputPrivate {
    pub output: OutputWitness,
    pub path: Vec<merkle::PathNode>,
}

impl PartialTxOutputPrivate {
    pub fn output_root(&self) -> [u8; 32] {
        let leaf = merkle::leaf(&self.output.commit().to_bytes());
        merkle::path_root(leaf, &self.path)
    }
}
