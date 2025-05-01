# SurrealDB Benchmarks

This project contains a set of binaries designed to benchmark and compare the performance of SurrealDB and RocksDB.

## Binaries

### SurrealDB Binaries

1. **`surrealdb_select_all`**
    - Selecting all records from a SurrealDB database. It uses SurrealDB SQL: "SELECT * FROM block"

2. **`surrealdb_parent_to_child_graph`**
    - Measures the performance of querying child-to-parent relationships using SurrealDB graph model(child->parent edges)

3. **`surrealdb_parent_to_child_pointers`**
    - Measures the performance of querying child-to-parent relationships by querying parents by their key

### RocksDB Binaries

1. **`rocksdb_select_all`**
    - Benchmarks the performance of selecting all records from a RocksDB database.

2. **`rocksdb_parent_to_child`**
    - Measures the performance of querying child-to-parent relationships by querying parents by their key

## Usage
To run the benchmarks, you can use the following commands:

```bash
 ./run_benchmarks.sh
```