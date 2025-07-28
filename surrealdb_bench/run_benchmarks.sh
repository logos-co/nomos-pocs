#!/bin/bash

BUILD_MODE="release"

BIN_DIR="target/$BUILD_MODE"

BENCHES=(
  "surrealdb_select_all"
  "surrealdb_parent_to_child_graph"
  "surrealdb_parent_to_child_pointers"
  "rocksdb_select_all"
  "rocksdb_parent_to_child"
)

# Read blocks_count from the command-line argument or default to 1000000
BLOCKS_COUNT=${1:-1000000}

echo "Building all binaries..."
cargo build --$BUILD_MODE
if [ $? -ne 0 ]; then
  echo "Error: Build failed."
  exit 1
fi


echo "Running benchmarks with blocks_count=$BLOCKS_COUNT"
for BIN in "${BENCHES[@]}"; do
  "$BIN_DIR/$BIN" --blocks-count "$BLOCKS_COUNT"
  if [ $? -ne 0 ]; then
    echo "Error: $BIN failed to execute."
    exit 1
  fi
done

echo "All benchmarks executed successfully."