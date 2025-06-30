use criterion::{Criterion, Throughput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hint::black_box;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SimpleStruct {
    pub id: u32,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BinaryStruct {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LargeStruct {
    pub items: Vec<ItemStruct>,
    pub map: HashMap<String, ItemStruct>,
    pub nested: SimpleData,
    pub blob: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemStruct {
    pub name: String,
    pub values: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SimpleData {
    pub values: HashMap<String, String>,
    pub inner: InnerData,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InnerData {
    pub count: u32,
    pub flag: bool,
}

pub fn generate_simple_struct() -> SimpleStruct {
    SimpleStruct {
        id: 12345,
        value: 9876543210,
    }
}

pub fn generate_binary_struct() -> BinaryStruct {
    BinaryStruct {
        data: vec![1, 2, 3, 4, 5],
    }
}

pub fn generate_large_struct() -> LargeStruct {
    let item1 = ItemStruct {
        name: "item_one".to_string(),
        values: vec![10, 20, 30],
    };

    let item2 = ItemStruct {
        name: "item_two".to_string(),
        values: vec![40, 50, 60],
    };

    let mut map = HashMap::new();
    map.insert("first".to_string(), item1.clone());
    map.insert("second".to_string(), item2.clone());

    LargeStruct {
        items: vec![item1, item2],
        map,
        nested: SimpleData {
            values: [
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]
            .into_iter()
            .collect(),
            inner: InnerData {
                count: 5000,
                flag: true,
            },
        },
        blob: vec![0u8; 512],
    }
}

pub trait EncodingBenchmark<T> {
    fn name() -> &'static str;
    fn encode(data: &T) -> Vec<u8>;
    fn decode(data: &[u8]) -> T;
}

pub fn bench_roundtrip<T, F>(c: &mut Criterion, data: &[T], test_name: &str)
where
    F: EncodingBenchmark<T>,
    T: Clone,
{
    let mut group = c.benchmark_group(format!("roundtrip_{}", test_name));
    group.throughput(Throughput::Elements(data.len() as u64));

    group.bench_function(F::name(), |b| {
        b.iter(|| {
            for item in data {
                let encoded = F::encode(black_box(item));
                let decoded = F::decode(black_box(&encoded));
                black_box(decoded);
            }
        })
    });

    group.finish();

    let sample_encoded = F::encode(&data[0]);
    println!(
        "{} {} - {} bytes per item",
        F::name(),
        test_name,
        sample_encoded.len()
    );
}
