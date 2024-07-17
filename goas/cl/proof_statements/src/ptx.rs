use cl::{merkle, InputWitness, OutputWitness, PtxRoot};
use serde::{Deserialize, Serialize};
/// An input to a partial transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialTxInputPrivate {
    pub input: InputWitness,
    pub cm_path: Vec<merkle::PathNode>,
    pub ptx_path: Vec<merkle::PathNode>,
}

impl PartialTxInputPrivate {
    pub fn ptx_root(&self) -> PtxRoot {
        let leaf = merkle::leaf(&self.input.commit().to_bytes());
        PtxRoot(merkle::path_root(leaf, &self.ptx_path))
    }

    pub fn cm_root(&self) -> [u8; 32] {
        let leaf = merkle::leaf(self.input.to_output_witness().commit_note().as_bytes());
        merkle::path_root(leaf, &self.cm_path)
    }
}

/// An output to a partial transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialTxOutputPrivate {
    pub output: OutputWitness,
    pub ptx_path: Vec<merkle::PathNode>,
}

impl PartialTxOutputPrivate {
    pub fn ptx_root(&self) -> PtxRoot {
        let leaf = merkle::leaf(&self.output.commit().to_bytes());
        PtxRoot(merkle::path_root(leaf, &self.ptx_path))
    }
}
