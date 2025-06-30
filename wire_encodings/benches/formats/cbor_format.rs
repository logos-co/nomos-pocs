use crate::common::EncodingBenchmark;
use serde::{Deserialize, Serialize};

pub struct CborFormat;
impl<T: Serialize + for<'de> Deserialize<'de>> EncodingBenchmark<T> for CborFormat {
    fn name() -> &'static str {
        "CBOR"
    }
    fn encode(data: &T) -> Vec<u8> {
        serde_cbor::to_vec(data).unwrap()
    }
    fn decode(data: &[u8]) -> T {
        serde_cbor::from_slice(data).unwrap()
    }
}
