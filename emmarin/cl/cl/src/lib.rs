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
    bincode::serde::encode_to_vec(data, bincode::config::standard()).unwrap()
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> T {
    let (value, bytes_read) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .expect("failed to deserialize");
    assert_eq!(bytes_read, bytes.len());
    value
}
