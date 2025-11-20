use crate::common::EncodingBenchmark;
use serde::{Deserialize, Serialize};

pub struct JsonFormat;
impl<T: Serialize + for<'de> Deserialize<'de>> EncodingBenchmark<T> for JsonFormat {
    fn name() -> &'static str {
        "JSON"
    }
    fn encode(data: &T) -> Vec<u8> {
        serde_json::to_vec(data).unwrap()
    }
    fn decode(data: &[u8]) -> T {
        serde_json::from_slice(data).unwrap()
    }
}
