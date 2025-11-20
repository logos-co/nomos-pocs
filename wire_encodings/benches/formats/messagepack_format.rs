use crate::common::EncodingBenchmark;
use serde::{Deserialize, Serialize};

pub struct MessagePackFormat;
impl<T: Serialize + for<'de> Deserialize<'de>> EncodingBenchmark<T> for MessagePackFormat {
    fn name() -> &'static str {
        "MessagePack"
    }
    fn encode(data: &T) -> Vec<u8> {
        rmp_serde::to_vec(data).unwrap()
    }
    fn decode(data: &[u8]) -> T {
        rmp_serde::from_slice(data).unwrap()
    }
}
