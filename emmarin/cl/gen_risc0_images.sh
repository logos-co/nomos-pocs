#!/usr/bin/env bash
set -e

# We generate in a *loop* because some risc0 proofs are recursive, so if a child
# proof's id changes, then the parent proof will also change, but we don't see the
# parent's id change until the next run.

cargo run --bin gen_risc0_images > risc0_images/src/lib.rs.new

while ! cmp -s risc0_images/src/lib.rs.new risc0_images/src/lib.rs
do
  mv risc0_images/src/lib.rs.new risc0_images/src/lib.rs
  cargo run --bin gen_risc0_images > risc0_images/src/lib.rs.new
  echo "-------- FINISHED UPDATE ITERATION --------"
done

rm risc0_images/src/lib.rs.new

cargo test -p risc0_images_police
