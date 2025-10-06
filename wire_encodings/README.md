# Choosing the Right Encoding for Networking

## 1. Introduction

Nomos currently uses **bincode** for all serialization. While fast and simple, bincode lacks support for **schema evolution** and **cross-language compatibility**—two features increasingly important as the ecosystem scales.

## 2. Selection Criteria

We evaluated candidate encoding formats based on:

* **Schema Evolution** – Supports changes to data structures without breaking compatibility.
* **Security** – Handles untrusted inputs safely.
* **Performance** – Fast to serialize and deserialize.
* **Determinism** – Always produces identical output for the same input.
* **Cross-language Compatibility** – Available and maintained across multiple languages.

## 3. Format Comparison (P2P-Relevant Formats)

| Encoding Format | Evolution | Deterministic | Security | Lang Support | Usage                 |
| --------------- | --------- | ------------- | -------- | ------------ | --------------------- |
| **Protobuf**    | ✓         | △             | ⬤⬤◯      | ⬤⬤⬤          | Cosmos SDK, libp2p    |
| **Borsh**       | —         | ✓             | ⬤⬤⬤      | ⬤⬤◯          | NEAR, Solana programs |
| **SCALE**       | —         | ✓             | ⬤⬤◯      | ⬤◯◯          | Polkadot              |
| **Bincode**     | —         | ✓             | ⬤⬤◯      | ⬤◯◯          | Solana validators     |
| **CBOR**        | ✓         | △             | ⬤⬤◯      | ⬤⬤⬤          | Cardano, IPFS         |
| **MsgPack**     | △         | △             | ⬤◯◯      | ⬤⬤⬤          | Algorand              |
| **SSZ**         | —         | ✓             | ⬤⬤⬤      | ⬤◯◯          | Ethereum 2.0          |

## 4. Tooling Considerations

### Serde-Generate

[https://crates.io/crates/serde-generate](https://crates.io/crates/serde-generate)

Auto-generates serializers for Rust types, but fails with complex constructs like `Risc0LeaderProof` due to issues with the `tracing` feature. Adding schemas manually is also non-trivial.

### Canonical Protobuf in Rust

[https://crates.io/crates/prost](https://crates.io/crates/prost)

We can use `prost` with deterministic configurations:

```rust
// In build.rs - use BTreeMap for consistent key ordering
let mut config = prost_build::Config::new();
config.btree_map(&["."]); // Apply to all map fields
config.compile_protos(&["your.proto"], &["."])?;
```

Manual canonicalization wrapper:

```rust
use prost::Message;

fn encode_with_prost<T: Message>(msg: &T) -> Vec<u8> {
    // Standard prost encoding. Output is deterministic if:
    // - map fields use BTreeMap via build config
    // - .proto field tag numbers match declaration order
    // - unknown or unset optional fields are avoided
    let mut buf = Vec::new();
    msg.encode(&mut buf).unwrap();
    buf
}
```

## 5. Why Evolution Matters

```rust
// V1
struct Block { height: u64, hash: [u8; 32] }

// V2
struct Block { height: u64, hash: [u8; 32], timestamp: u64 }
```

Rigid encodings like bincode break when structures evolve. Nomos may support thousands of clients, making coordinated upgrades costly. Formats that support optional fields and schema evolution reduce this friction.

## 6. Planning for Multi-Language Support

Support for multiple implementations matters most post-adoption. Initially, most ecosystems rely on one client. However, protocol-level encoding should avoid locking into a single ecosystem from the start.

## 7. Serialization Benchmark Analysis

### Benchmark Overview

This analysis compares 8 serialization formats: Bincode, Borsh, CBOR, JSON, MessagePack, Protobuf, SCALE, and SSZ. The benchmarks test performance on different data types and scales.

### Top Observations

* Bincode offers best performance for small and medium-sized data
* SCALE and Borsh perform best on large structures
* Protobuf offers compact encoding size and moderate throughput

### Simple Structs Serialization (Roundtrip Performance)

| Encoding Format | Roundtrip Time | Size (bytes) | Throughput (Melem/s) |
| --------------- | -------------- | ------------ | -------------------- |
| Bincode         | 607.66 ns      | 12           | 82.28                |
| Borsh           | 775.14 ns      | 12           | 64.51                |
| SSZ             | 947.31 ns      | 12           | 52.78                |
| SCALE           | 1.06 µs        | 10           | 47.31                |
| Protobuf        | 1.21 µs        | 9            | 41.24                |
| MessagePack     | 2.12 µs        | 13           | 23.63                |
| JSON            | 3.74 µs        | 31           | 13.36                |
| CBOR            | 5.70 µs        | 22           | 8.77                 |

### Binary Structs Performance

| Encoding Format | Roundtrip Time | Size (bytes) | Throughput (Melem/s) |
| --------------- | -------------- | ------------ | -------------------- |
| Bincode         | 1.59 µs        | 13           | 31.55                |
| Borsh           | 1.81 µs        | 9            | 27.64                |
| SCALE           | 1.83 µs        | 6            | 27.25                |
| Protobuf        | 2.66 µs        | 7            | 18.81                |
| MessagePack     | 3.19 µs        | 7            | 15.66                |
| SSZ             | 4.27 µs        | 9            | 11.70                |
| CBOR            | 4.82 µs        | 12           | 10.37                |
| JSON            | 4.99 µs        | 20           | 10.03                |

### Large Structs Performance

| Encoding Format | Roundtrip Time | Size (bytes) | Throughput (Kelem/s) |
| --------------- | -------------- | ------------ | -------------------- |
| SCALE           | 2.85 µs        | 647          | 1,755                |
| Borsh           | 3.05 µs        | 700          | 1,637                |
| Protobuf        | 4.81 µs        | 643          | 1,041                |
| Bincode         | 5.29 µs        | 772          | 946                  |
| SSZ             | 8.57 µs        | 589          | 583                  |
| CBOR            | 18.13 µs       | 720          | 276                  |
| MessagePack     | 18.45 µs       | 618          | 271                  |
| JSON            | 27.11 µs       | 1,318        | 184                  |

### Performance Scaling Characteristics

| Format      | Simple Structs | Binary Structs | Large Structs | Overall Scalability |
| ----------- | -------------- | -------------- | ------------- | ------------------- |
| Bincode     | ⬤⬤⬤            | ⬤⬤⬤            | ⬤⬤◯           | Strong              |
| SCALE       | ⬤⬤◯            | ⬤⬤⬤            | ⬤⬤⬤           | Strong              |
| Borsh       | ⬤⬤◯            | ⬤⬤◯            | ⬤⬤⬤           | Strong              |
| Protobuf    | ⬤⬤◯            | ⬤⬤◯            | ⬤⬤◯           | Good                |
| SSZ         | ⬤◯◯            | ⬤◯◯            | ⬤◯◯           | Good                |
| MessagePack | ⬤◯◯            | ⬤◯◯            | ⬤◯◯           | Good                |
| CBOR        | ⬤◯◯            | ⬤⬤◯            | ⬤◯◯           | Moderate            |
| JSON        | ⬤◯◯            | ⬤◯◯            | ⬤◯◯           | Moderate            |

## 8. Encoding Evaluation and Conclusion

Benchmark results show meaningful performance differences across formats, especially in large and nested structures. While bincode leads in raw speed for small data, SCALE and Borsh outperform others in large-structure throughput.

**Good alternatives to consider:**

* **Protobuf** – Supports schema evolution, great language support, and offers compact size with moderate performance.
* **Borsh and SCALE** – Compact and deterministic; good performance, but lack schema evolution and broader tooling.

Each has trade-offs. The decision depends on the need for schema evolution, performance, cross-language support, and implementation simplicity.

## 9. How to run benchmarks
To run the benchmarks, clone the repository and execute:

```bash
cargo bench
```
