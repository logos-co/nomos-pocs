#!/usr/bin/env bash
set -e

# Order of these proofs is important, we sequence them topologically by composition order
proofs=$(cat <<EOF
risc0_proofs
bundle_risc0_proof
ledger_risc0_proof
EOF
)

for proof in $proofs; do
    echo "Building $proof"
    # Run the cargo risczero build command and process output line by line
    cargo risczero build --manifest-path "$proof/Cargo.toml" | while read -r line; do
        # Parse out the 
        if [[ $line =~ ImageID:\ ([0-9a-f]+)\ -\ (.+\.bin) ]]; then
            image_id="${BASH_REMATCH[1]}"
            image_elf="${BASH_REMATCH[2]}"
            image_name=$(basename $image_elf .bin | tr '[:lower:]' '[:upper:]')
            echo "${image_name}_ID: $image_id"
            echo "${image_name}_ELF: $(cat $image_elf | xxd -p | head -c 32)..."

            echo $image_id | tr -d '\n' > "risc0_images/src/${image_name}_ID"
            cp $image_elf "risc0_images/src/${image_name}_ELF"
        fi
    done
done
