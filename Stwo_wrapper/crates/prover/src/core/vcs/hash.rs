use std::fmt::{Debug, Display};

pub trait Hash:
    Copy
    + Default
    + Display
    + Debug
    + Eq
    + Send
    + Sync
    + 'static
{
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}