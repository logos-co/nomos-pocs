use crate::common::{BinaryStruct, EncodingBenchmark, LargeStruct, SimpleStruct};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SimpleStructBorsh {
    pub id: u32,
    pub value: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct BinaryStructBorsh {
    pub data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ItemStructBorsh {
    pub name: String,
    pub values: Vec<u32>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SimpleDataBorsh {
    pub values: std::collections::BTreeMap<String, String>,
    pub inner: InnerDataBorsh,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct InnerDataBorsh {
    pub count: u32,
    pub flag: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct LargeStructBorsh {
    pub items: Vec<ItemStructBorsh>,
    pub map: std::collections::BTreeMap<String, ItemStructBorsh>,
    pub nested: SimpleDataBorsh,
    pub blob: Vec<u8>,
}

pub fn convert_to_borsh_simple(data: &SimpleStruct) -> SimpleStructBorsh {
    SimpleStructBorsh {
        id: data.id,
        value: data.value,
    }
}

pub fn convert_to_borsh_binary(data: &BinaryStruct) -> BinaryStructBorsh {
    BinaryStructBorsh {
        data: data.data.clone(),
    }
}

pub fn convert_to_borsh_large(data: &LargeStruct) -> LargeStructBorsh {
    LargeStructBorsh {
        items: data
            .items
            .iter()
            .map(|item| ItemStructBorsh {
                name: item.name.clone(),
                values: item.values.clone(),
            })
            .collect(),
        map: data
            .map
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    ItemStructBorsh {
                        name: v.name.clone(),
                        values: v.values.clone(),
                    },
                )
            })
            .collect(),
        nested: SimpleDataBorsh {
            values: data
                .nested
                .values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            inner: InnerDataBorsh {
                count: data.nested.inner.count,
                flag: data.nested.inner.flag,
            },
        },
        blob: data.blob.clone(),
    }
}

pub struct BorshFormat;
impl<T: BorshSerialize + BorshDeserialize> EncodingBenchmark<T> for BorshFormat {
    fn name() -> &'static str {
        "Borsh"
    }
    fn encode(data: &T) -> Vec<u8> {
        borsh::to_vec(data).unwrap()
    }
    fn decode(data: &[u8]) -> T {
        borsh::from_slice(data).unwrap()
    }
}
