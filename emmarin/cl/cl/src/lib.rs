pub mod crust;
pub mod ds;
pub mod mantle;
pub mod merkle;

pub type Hash = risc0_zkvm::sha::rust_crypto::Sha256;
pub use digest::Digest;

pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hash::new();
    hasher.update(data);
    hasher.finalize().into()
}
