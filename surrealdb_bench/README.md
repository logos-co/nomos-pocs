# SurrealDB Benchmarks

This project contains a set of benches to benchmark and compare the performance of SurrealDB and RocksDB.

## Benches

### SurrealDB Binaries

1. **`surrealdb_select_all`**
    - Selecting all records from a SurrealDB database.

2. **`surrealdb_parent_to_child_graph`**
    - Measures the performance of querying child-to-parent relationships using SurrealDB graph model(child->parent edges)

3. **`surrealdb_parent_to_child_pointers`**
    - Measures the performance of querying child-to-parent relationships by querying parents by their key

### RocksDB Binaries

1. **`rocksdb_select_all`**
    - Benchmarks the performance of selecting all records

2. **`rocksdb_parent_to_child`**
    - Measures the performance of querying child-to-parent relationships by querying parents by their key

## Usage
To run the benchmarks, you can use the following command:

```bash
 ./run_benchmarks.sh [BLOCKS_COUNT]
```

Default `BLOCKS_COUNT` is 1000000