use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

mod common;
mod formats;

use common::*;
use formats::*;

fn benchmark_simple_structs(c: &mut Criterion) {
    let data: Vec<SimpleStruct> = (0..50).map(|_| generate_simple_struct()).collect();
    let borsh_data: Vec<_> = data.iter().map(convert_to_borsh_simple).collect();
    let scale_data: Vec<_> = data.iter().map(convert_to_scale_simple).collect();
    let ssz_data: Vec<_> = data.iter().map(convert_to_ssz_simple).collect();
    let proto_data: Vec<_> = data.iter().map(convert_to_proto_simple).collect();

    println!("\n=== SIMPLE STRUCTS COMPARISON ===");
    bench_roundtrip::<SimpleStruct, BincodeFormat>(c, &data, "simple");
    bench_roundtrip::<SimpleStruct, JsonFormat>(c, &data, "simple");
    bench_roundtrip::<SimpleStruct, CborFormat>(c, &data, "simple");
    bench_roundtrip::<SimpleStruct, MessagePackFormat>(c, &data, "simple");
    bench_roundtrip::<borsh_format::SimpleStructBorsh, BorshFormat>(c, &borsh_data, "simple");
    bench_roundtrip::<scale_format::SimpleStructScale, ScaleFormat>(c, &scale_data, "simple");
    bench_roundtrip::<ssz_format::SimpleStructSsz, SszFormat>(c, &ssz_data, "simple");
    bench_roundtrip::<protobuf_format::SimpleStructProto, ProtobufFormat>(c, &proto_data, "simple");
}

fn benchmark_binary_structs(c: &mut Criterion) {
    let data: Vec<BinaryStruct> = (0..50).map(|_| generate_binary_struct()).collect();
    let borsh_data: Vec<_> = data.iter().map(convert_to_borsh_binary).collect();
    let scale_data: Vec<_> = data.iter().map(convert_to_scale_binary).collect();
    let ssz_data: Vec<_> = data.iter().map(convert_to_ssz_binary).collect();
    let proto_data: Vec<_> = data.iter().map(convert_to_proto_binary).collect();

    println!("\n=== BINARY STRUCTS COMPARISON ===");
    bench_roundtrip::<BinaryStruct, BincodeFormat>(c, &data, "binary");
    bench_roundtrip::<BinaryStruct, JsonFormat>(c, &data, "binary");
    bench_roundtrip::<BinaryStruct, CborFormat>(c, &data, "binary");
    bench_roundtrip::<BinaryStruct, MessagePackFormat>(c, &data, "binary");
    bench_roundtrip::<borsh_format::BinaryStructBorsh, BorshFormat>(c, &borsh_data, "binary");
    bench_roundtrip::<scale_format::BinaryStructScale, ScaleFormat>(c, &scale_data, "binary");
    bench_roundtrip::<ssz_format::BinaryStructSsz, SszFormat>(c, &ssz_data, "binary");
    bench_roundtrip::<protobuf_format::BinaryStructProto, ProtobufFormat>(c, &proto_data, "binary");
}

fn benchmark_large_structs(c: &mut Criterion) {
    let data: Vec<LargeStruct> = (0..5).map(|_| generate_large_struct()).collect();
    let borsh_data: Vec<_> = data.iter().map(convert_to_borsh_large).collect();
    let scale_data: Vec<_> = data.iter().map(convert_to_scale_large).collect();
    let ssz_data: Vec<_> = data.iter().map(convert_to_ssz_large).collect();
    let proto_data: Vec<_> = data.iter().map(convert_to_proto_large).collect();

    println!("\n=== LARGE STRUCTS COMPARISON ===");
    bench_roundtrip::<LargeStruct, BincodeFormat>(c, &data, "large");
    bench_roundtrip::<LargeStruct, JsonFormat>(c, &data, "large");
    bench_roundtrip::<LargeStruct, CborFormat>(c, &data, "large");
    bench_roundtrip::<LargeStruct, MessagePackFormat>(c, &data, "large");
    bench_roundtrip::<borsh_format::LargeStructBorsh, BorshFormat>(c, &borsh_data, "large");
    bench_roundtrip::<scale_format::LargeStructScale, ScaleFormat>(c, &scale_data, "large");
    bench_roundtrip::<ssz_format::LargeStructSsz, SszFormat>(c, &ssz_data, "large");
    bench_roundtrip::<protobuf_format::LargeStructProto, ProtobufFormat>(c, &proto_data, "large");
}

criterion_group!(
    name = combined_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        benchmark_simple_structs,
        benchmark_binary_structs,
        benchmark_large_structs
);

criterion_main!(combined_benches);
