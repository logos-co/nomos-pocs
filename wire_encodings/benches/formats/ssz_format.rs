use crate::common::{BinaryStruct, EncodingBenchmark, LargeStruct, SimpleStruct};
use ssz::{Decode as SszDecode, Encode as SszEncode};
use ssz_derive::{Decode as SszDecode, Encode as SszEncode};

#[derive(SszEncode, SszDecode, Debug, Clone, PartialEq)]
pub struct SimpleStructSsz {
    pub id: u32,
    pub value: u64,
}

#[derive(SszEncode, SszDecode, Debug, Clone, PartialEq)]
pub struct BinaryStructSsz {
    pub data: Vec<u8>,
}

#[derive(SszEncode, SszDecode, Debug, Clone, PartialEq)]
pub struct ItemStructSsz {
    pub name: Vec<u8>,
    pub values: Vec<u32>,
}

#[derive(SszEncode, SszDecode, Debug, Clone, PartialEq)]
pub struct InnerDataSsz {
    pub count: u32,
    pub flag: bool,
}

#[derive(SszEncode, SszDecode, Debug, Clone, PartialEq)]
pub struct LargeStructSsz {
    pub items: Vec<ItemStructSsz>,
    pub nested: InnerDataSsz,
    pub blob: Vec<u8>,
}

pub fn convert_to_ssz_simple(data: &SimpleStruct) -> SimpleStructSsz {
    SimpleStructSsz {
        id: data.id,
        value: data.value,
    }
}

pub fn convert_to_ssz_binary(data: &BinaryStruct) -> BinaryStructSsz {
    BinaryStructSsz {
        data: data.data.clone(),
    }
}

pub fn convert_to_ssz_large(data: &LargeStruct) -> LargeStructSsz {
    LargeStructSsz {
        items: data
            .items
            .iter()
            .map(|item| ItemStructSsz {
                name: item.name.as_bytes().to_vec(),
                values: item.values.clone(),
            })
            .collect(),
        nested: InnerDataSsz {
            count: data.nested.inner.count,
            flag: data.nested.inner.flag,
        },
        blob: data.blob.clone(),
    }
}

pub struct SszFormat;
impl<T: SszEncode + SszDecode> EncodingBenchmark<T> for SszFormat {
    fn name() -> &'static str {
        "SSZ"
    }
    fn encode(data: &T) -> Vec<u8> {
        data.as_ssz_bytes()
    }
    fn decode(data: &[u8]) -> T {
        T::from_ssz_bytes(data).unwrap()
    }
}
