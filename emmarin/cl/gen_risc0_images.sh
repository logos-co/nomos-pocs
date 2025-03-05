#!/usr/bin/env bash
set -e

# We generate in a *loop* because some risc0 proofs are recursive, so if a child
# proof's id changes, then the parent proof will also change, but we don't see the
# parent's id change until the next run.

proofs=$(cat <<EOF
tx_risc0_proof/tx
risc0_proofs/stf_nop
ledger_validity_proof/ledger
bundle_risc0_proof/bundle
EOF
)

for proof in $proofs; do
    echo "Building $proof"
    # Run the cargo risczero build command and process output line by line
    cargo risczero build --manifest-path "$proof/Cargo.toml" | while read -r line; do
        # Parse out the 
        if [[ $line =~ ImageID:\ ([0-9a-f]+)\ -\ \"(.+)\" ]]; then
            image_id="${BASH_REMATCH[1]}"
            image_elf="${BASH_REMATCH[2]}"
            image_name=$(basename $image_elf | tr '[:lower:]' '[:upper:]')
            echo "ID: $image_id"
            echo "ELF: $image_elf"

            cp $image_elf "risc0_images/src/${image_name}_ELF"
            echo $image_id > "risc0_images/src/${image_name}_ID"
        fi
    done
done
exit


cargo run --release --bin gen_risc0_images > risc0_images/src/lib.rs.new

while ! cmp -s risc0_images/src/lib.rs.new risc0_images/src/lib.rs
do
    echo "FOLLOWING PROOF IDS HAVE CHANGED:"
    diff risc0_images/src/lib.rs.new risc0_images/src/lib.rs  | rg '_ID' | grep "^<" | while read line; do id=$(echo "$line" | grep -o "[A-Z_]*_ID"); if [ -n "$id" ]; then hash=$(echo -n "$line" | sha256sum | cut -d" " -f1); echo " - $id  ${hash:0:5}"; fi; done

    echo "FOLLOWING PROOF ELFS HAVE CHANGED:"
    diff risc0_images/src/lib.rs.new risc0_images/src/lib.rs  | rg '_ELF' | grep "^<" | while read line; do elf=$(echo "$line" | grep -o "[A-Z_]*_ELF"); if [ -n "$elf" ]; then hash=$(echo -n "$line" | sha256sum | cut -d" " -f1); echo " - $elf ${hash:0:5}"; fi; done


    mv risc0_images/src/lib.rs.new risc0_images/src/lib.rs
    cargo run --release --bin gen_risc0_images > risc0_images/src/lib.rs.new
    echo "-------- FINISHED UPDATE ITERATION --------"
done

rm risc0_images/src/lib.rs.new

cargo test -p risc0_images_police
