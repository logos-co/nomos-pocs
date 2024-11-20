use cl::cl::{
    merkle,
    pact::{Pact, PactWitness},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PactPublic {
    pub pact: Pact,
    pub cm_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PactPrivate {
    pub pact: PactWitness,
    pub input_cm_paths: Vec<Vec<merkle::PathNode>>,
    pub cm_root: [u8; 32],
}
