# Nomos Storage Benchmarks

Goal: tune RocksDB for Nomos validator workloads using realistic data and sizes. The approach is to run benchmarks and try different parameters and settings, then compare results.

## What it does

- Generates datasets that approximate realistic sizes and access patterns.
- Runs mixed read/write validator-style workloads against RocksDB.
- Varies RocksDB parameters (cache, write buffer, compaction, block size, compression).
- Records throughput and basic variability across repeated runs.

## Quick start

1) Generate a dataset
```bash
POL_PROOF_DEV_MODE=true RUST_LOG=info cargo run --bin dataset_generator -- --config dataset_configs/annual_mainnet.toml
```

2) Run a baseline
```bash
RUST_LOG=info cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120
```

3) Try parameters and compare
```bash
# Cache size
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --cache-size 25
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --cache-size 40
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --cache-size 55

# Write buffer (use the best cache size observed)
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --cache-size 40 --write-buffer 128
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --cache-size 40 --write-buffer 256

# Compaction jobs
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --cache-size 40 --write-buffer 128 --compaction-jobs 8
```

## How to evaluate

- One warmup and at least three measured runs per setting.
- Fixed seed when exact reproducibility is required.
- Compare mean ops/sec and variability across runs.
- Change one setting at a time.

## Parameter ranges under evaluation

- Block cache: 25–55% of RAM
- Write buffer: 64–256 MB
- Compaction jobs: 4–12
- Block size: 16–64 KB
- Compression: none, lz4, snappy, zstd

## Profiles and datasets

Validator profiles:
- light (~100 validators)
- mainnet (~2000 validators)
- testnet (~1000 validators)

Datasets:
- quick_test.toml: ~27 MB (fast checks)
- testnet_sim.toml: ~1 GB
- annual_mainnet.toml: ~40 GB

## CLI
```bash
cargo run --bin storage_bench_runner -- [OPTIONS]

--profile          light | mainnet | testnet
--memory           RAM limit in GB (default: 8)
--duration         Benchmark duration (default: 120)
--cache-size       Block cache size as % of RAM (20–60)
--write-buffer     Write buffer size in MB (64–512)
--compaction-jobs  Background compaction jobs (4–16)
--block-size       Table block size in KB (8–64)
--compression      none | lz4 | snappy | zstd
--seed             RNG seed
--warmup-runs      Warmup iterations (default: 1)
--measurement-runs Measurement iterations (default: 3)
--read-only        Read-only mode
```

Reproducible run:
```bash
cargo run --bin storage_bench_runner -- --profile mainnet --memory 8 --duration 120 --seed 12345
```

## Test plan

Purpose: verify that benchmarks run, produce results, and that parameter changes have measurable effects.

### Scope

- Dataset generation at different sizes.
- Benchmark runs across profiles.
- Parameter sweeps for cache, write buffer, compaction, block size, compression.
- Result capture (JSON) and basic summary output.

### Environments

- Memory limits: 4 GB, 8 GB, 16 GB.
- Datasets: small (quick), medium, large.
- Duration: short for exploration (60–120s), longer to confirm (180–300s).

### Test cases

1. Dataset generation
   - Small dataset completes.
   - Large dataset resumes if partially present.
   - Outputs stored in expected path.

2. Baseline benchmark
   - Runs with selected profile and memory limit.
   - Produces JSON results and console summary.

3. Cache size
   - 25%, 40%, 55%.
   - Compare mean ops/sec and variability.
   - Record chosen value.

4. Write buffer
   - Keep chosen cache.
   - 128 MB, 256 MB (and 64/512 MB if needed).
   - Record impact, pick value.

5. Compaction jobs
   - 4, 8, 12 (or within system limits).
   - Check for stalls or CPU saturation.

6. Block size
   - 16 KB, 32 KB, 64 KB.
   - Evaluate read performance and variability.

7. Compression
   - none, lz4, snappy, zstd.
   - Compare throughput; consider disk footprint if relevant.

8. Reproducibility
   - Repeat a chosen run with a fixed seed.
   - Confirm similar results across iterations.

9. Memory sensitivity
   - Re-run chosen settings at lower and higher memory limits.
   - Check for regressions.

### Acceptance criteria

- All runs complete without errors.
- Results are saved (JSON present).
- Chosen settings show a measurable improvement over baseline.
- Variability remains acceptable for this use case.

### Reporting

- Log command lines and seeds used.
- Note dataset, profile, memory, and duration for each run.
- Store JSON result files together for comparison.

## Outputs

- Datasets: ~/.nomos_storage_benchmarks/rocksdb_data
- Results (JSON): ~/.nomos_storage_benchmarks/results/
- Console summary shows mean ops/sec and variability.

## Requirements

- Rust 1.75+
- 8+ GB RAM (more for larger datasets)
- ~50+ GB disk for the largest dataset

## Notes

- Baseline first, then change one parameter at a time.
- Keep runs short while exploring; confirm with longer runs when needed.

## Why no general-purpose benchmarking library

- Workloads require long-running mixed operations (reads, range scans, writes) against a prebuilt dataset; typical micro-benchmark frameworks focus on short, isolated functions.
- We need control over dataset size/layout, memory limits, and external RocksDB options; this is easier with a purpose-built runner.
- Results include per-run JSON with config and summary metrics; integrating this into a generic harness would add overhead without benefit here.