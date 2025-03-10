pub mod crust;
pub mod ds;
pub mod mantle;

pub use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};
use serde::{de::DeserializeOwned, Serialize};

pub type Hash = Sha256;

pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hash::new();
    hasher.update(data);
    hasher.finalize().into()
}

// TODO: spec serializiation
pub fn serialize(data: impl Serialize) -> Vec<u8> {
    bincode::serialize(&data).unwrap()
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> T {
    bincode::deserialize(bytes).unwrap()
}
