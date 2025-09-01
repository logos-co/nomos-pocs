use crate::common::{BinaryStruct, EncodingBenchmark, LargeStruct, SimpleStruct};
use prost::Message;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Message)]
pub struct SimpleStructProto {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(uint64, tag = "2")]
    pub value: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct BinaryStructProto {
    #[prost(bytes = "vec", tag = "1")]
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ItemStructProto {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(uint32, repeated, tag = "2")]
    pub values: Vec<u32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct InnerDataProto {
    #[prost(uint32, tag = "1")]
    pub count: u32,
    #[prost(bool, tag = "2")]
    pub flag: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct SimpleDataProto {
    #[prost(map = "string, string", tag = "1")]
    pub values: HashMap<String, String>,
    #[prost(message, optional, tag = "2")]
    pub inner: Option<InnerDataProto>,
}

#[derive(Clone, PartialEq, Message)]
pub struct LargeStructProto {
    #[prost(message, repeated, tag = "1")]
    pub items: Vec<ItemStructProto>,
    #[prost(map = "string, message", tag = "2")]
    pub map: HashMap<String, ItemStructProto>,
    #[prost(message, optional, tag = "3")]
    pub nested: Option<SimpleDataProto>,
    #[prost(bytes = "vec", tag = "4")]
    pub blob: Vec<u8>,
}

pub fn convert_to_proto_simple(data: &SimpleStruct) -> SimpleStructProto {
    SimpleStructProto {
        id: data.id,
        value: data.value,
    }
}

pub fn convert_to_proto_binary(data: &BinaryStruct) -> BinaryStructProto {
    BinaryStructProto {
        data: data.data.clone(),
    }
}

pub fn convert_to_proto_large(data: &LargeStruct) -> LargeStructProto {
    LargeStructProto {
        items: data
            .items
            .iter()
            .map(|item| ItemStructProto {
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
                    ItemStructProto {
                        name: v.name.clone(),
                        values: v.values.clone(),
                    },
                )
            })
            .collect(),
        nested: Some(SimpleDataProto {
            values: data.nested.values.clone(),
            inner: Some(InnerDataProto {
                count: data.nested.inner.count,
                flag: data.nested.inner.flag,
            }),
        }),
        blob: data.blob.clone(),
    }
}

pub struct ProtobufFormat;
impl<T: Message + Default> EncodingBenchmark<T> for ProtobufFormat {
    fn name() -> &'static str {
        "Protobuf"
    }
    fn encode(data: &T) -> Vec<u8> {
        data.encode_to_vec()
    }
    fn decode(data: &[u8]) -> T {
        T::decode(data).unwrap()
    }
}
