use crate::common::{BinaryStruct, EncodingBenchmark, LargeStruct, SimpleStruct};
use parity_scale_codec::{Decode as ScaleDecode, Encode as ScaleEncode};

#[derive(ScaleEncode, ScaleDecode, Debug, Clone, PartialEq)]
pub struct SimpleStructScale {
    #[codec(compact)]
    pub id: u32,
    pub value: u64,
}

#[derive(ScaleEncode, ScaleDecode, Debug, Clone, PartialEq)]
pub struct BinaryStructScale {
    pub data: Vec<u8>,
}

#[derive(ScaleEncode, ScaleDecode, Debug, Clone, PartialEq)]
pub struct ItemStructScale {
    pub name: Vec<u8>,
    pub values: Vec<u32>,
}

#[derive(ScaleEncode, ScaleDecode, Debug, Clone, PartialEq)]
pub struct SimpleDataScale {
    pub values: Vec<(Vec<u8>, Vec<u8>)>,
    pub inner: InnerDataScale,
}

#[derive(ScaleEncode, ScaleDecode, Debug, Clone, PartialEq)]
pub struct InnerDataScale {
    pub count: u32,
    pub flag: bool,
}

#[derive(ScaleEncode, ScaleDecode, Debug, Clone, PartialEq)]
pub struct LargeStructScale {
    pub items: Vec<ItemStructScale>,
    pub map: Vec<(Vec<u8>, ItemStructScale)>,
    pub nested: SimpleDataScale,
    pub blob: Vec<u8>,
}

pub fn convert_to_scale_simple(data: &SimpleStruct) -> SimpleStructScale {
    SimpleStructScale {
        id: data.id,
        value: data.value,
    }
}

pub fn convert_to_scale_binary(data: &BinaryStruct) -> BinaryStructScale {
    BinaryStructScale {
        data: data.data.clone(),
    }
}

pub fn convert_to_scale_large(data: &LargeStruct) -> LargeStructScale {
    LargeStructScale {
        items: data
            .items
            .iter()
            .map(|item| ItemStructScale {
                name: item.name.as_bytes().to_vec(),
                values: item.values.clone(),
            })
            .collect(),
        map: data
            .map
            .iter()
            .map(|(k, v)| {
                (
                    k.as_bytes().to_vec(),
                    ItemStructScale {
                        name: v.name.as_bytes().to_vec(),
                        values: v.values.clone(),
                    },
                )
            })
            .collect(),
        nested: SimpleDataScale {
            values: data
                .nested
                .values
                .iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v.as_bytes().to_vec()))
                .collect(),
            inner: InnerDataScale {
                count: data.nested.inner.count,
                flag: data.nested.inner.flag,
            },
        },
        blob: data.blob.clone(),
    }
}

pub struct ScaleFormat;
impl<T: ScaleEncode + ScaleDecode> EncodingBenchmark<T> for ScaleFormat {
    fn name() -> &'static str {
        "SCALE"
    }
    fn encode(data: &T) -> Vec<u8> {
        data.encode()
    }
    fn decode(data: &[u8]) -> T {
        T::decode(&mut &data[..]).unwrap()
    }
}
