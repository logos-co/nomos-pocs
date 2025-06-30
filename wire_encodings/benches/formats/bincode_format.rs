use crate::common::EncodingBenchmark;
use serde::{Deserialize, Serialize};

pub struct BincodeFormat;
impl<T: Serialize + for<'de> Deserialize<'de>> EncodingBenchmark<T> for BincodeFormat {
    fn name() -> &'static str {
        "Bincode"
    }
    fn encode(data: &T) -> Vec<u8> {
        bincode::serialize(data).unwrap()
    }
    fn decode(data: &[u8]) -> T {
        bincode::deserialize(data).unwrap()
    }
}
