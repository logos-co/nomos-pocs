pub mod crust;
pub mod ds;
pub mod mantle;

pub use risc0_zkvm::sha::rust_crypto::{Digest, Sha256};

pub type Hash = Sha256;

pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hash::new();
    hasher.update(data);
    hasher.finalize().into()
}
