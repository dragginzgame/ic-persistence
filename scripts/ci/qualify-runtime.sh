#!/usr/bin/env bash
# Installed consumer proof; Make prepares builds, Testkit owns server lifecycle.
set -euo pipefail
[[ $# == 5 ]] || exit 2
root="$1" cli="$2" server_directory="$3" optimizer="$4" ic_pins="$5"
cd "$root"
server="$("$cli" check --directory "$server_directory")"
mkdir -p target/qualification/runtime
evidence="$(mktemp -d target/qualification/runtime/attempt.XXXXXX)"
artifacts=(wasm_io_qualification: wasm_storage_qualification:.storage)
shasum -a 256 Cargo.toml Cargo.lock .shared-tooling.snapshot ci/runtime-tools.env "$ic_pins" \
    testing/runtime-qualification/Cargo.toml testing/runtime-qualification/Cargo.lock \
    target/runtime-qualification/debug/ic-memory-runtime-qualification "$cli" "$server" "$optimizer" > "$evidence/inputs.sha256"
for artifact in "${artifacts[@]}"; do
    IFS=: read -r stem suffix <<< "$artifact"
    shasum -a 256 "target/wasm32-unknown-unknown/wasm-size/examples/$stem.wasm" >> "$evidence/inputs.sha256"
done
for optimization in original O3 Os Oz; do
    for artifact in "${artifacts[@]}"; do
        IFS=: read -r stem suffix <<< "$artifact"
        wasm="$root/$evidence/$optimization$suffix.wasm"
        if [[ "$optimization" == original ]]; then
            cp "target/wasm32-unknown-unknown/wasm-size/examples/$stem.wasm" "$wasm"
        else
            "$optimizer" "$evidence/original$suffix.wasm" --enable-bulk-memory-opt "-$optimization" -o "$wasm" \
                > "$evidence/$optimization$suffix.optimizer.log" 2>&1 || {
                    status=$?; cat "$evidence/$optimization$suffix.optimizer.log"; exit "$status";
                }
        fi
        shasum -a 256 "$wasm" >> "$evidence/inputs.sha256"
    done
    echo "Runtime qualification: $optimization ($evidence)"
    status=0
    env -u POCKET_IC_BIN -u IC_TESTKIT_POCKET_IC_URL \
        IC_MEMORY_QUALIFICATION_WASM="$root/$evidence/$optimization.wasm" \
        IC_STORAGE_QUALIFICATION_WASM="$root/$evidence/$optimization.storage.wasm" \
        "$cli" run --directory "$server_directory" --ttl 900 \
        --server-stdout "$evidence/$optimization.server.stdout" --server-stderr "$evidence/$optimization.server.stderr" \
        -- "$root/target/runtime-qualification/debug/ic-memory-runtime-qualification" \
        > "$evidence/$optimization.runtime.log" 2>&1 || status=$?
    cat "$evidence/$optimization.runtime.log"
    [[ "$status" == 0 ]] || exit "$status"
done
cat "$evidence/inputs.sha256"
