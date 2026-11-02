#!/usr/bin/env bash
# Caller admits Testkit; this comparison never installs tools or updates locks.
set -euo pipefail
[[ $# == 5 ]] || exit 2
root="$1" cli="$2" server_directory="$3" rows="$4" samples="$5"
[[ "$rows" =~ ^[1-9][0-9]*$ && ${#rows} -le 5 && "$rows" -le 16384 ]]
[[ "$samples" =~ ^[1-9][0-9]*$ && ${#samples} -le 2 && "$samples" -le 20 ]]
cd "$root"
export RUSTUP_AUTO_INSTALL=0
server="$("$cli" check --directory "$server_directory")"
mkdir -p target/storage-comparison
output="$(mktemp -d "$root/target/storage-comparison/attempt.XXXXXX")"
printf 'Storage comparison evidence: %s\n' "$output"
manifest=testing/runtime-qualification/Cargo.toml
rustc -vV > "$output/compiler.txt"
cargo -V > "$output/cargo.txt"
cp "$manifest" testing/runtime-qualification/Cargo.lock "$output/"
cp Cargo.lock "$output/production.lock"
git rev-parse HEAD > "$output/source-commit"
git diff --binary > "$output/pending.patch"
# Hash actual working source, including untracked files from the pending hard cut.
rg --files crates/ic-storage testing/runtime-qualification/crates/ic-storage-benchmarks | LC_ALL=C sort > "$output/source-files"
while IFS= read -r path; do shasum -a 256 "$path"; done < "$output/source-files" > "$output/source.sha256"
tar -czf "$output/source.tar.gz" -T "$output/source-files" Cargo.toml rust-toolchain.toml "$manifest" scripts/ci/compare-storage.sh
shasum -a 256 "$cli" "$server" Cargo.toml rust-toolchain.toml "$manifest" testing/runtime-qualification/Cargo.lock > "$output/inputs.sha256"
for backend in owned dfinity; do
    if ! cargo build --locked --offline --manifest-path "$manifest" -p ic-storage-benchmarks \
        --example storage-comparison-wasm --target wasm32-unknown-unknown --profile comparison \
        --features "$backend" --target-dir "target/storage-comparison-build/$backend" > "$output/$backend-build.log" 2>&1; then
        cat "$output/$backend-build.log" >&2; exit 1
    fi
    cp "target/storage-comparison-build/$backend/wasm32-unknown-unknown/comparison/examples/storage_comparison_wasm.wasm" "$output/$backend.wasm"
done
if ! cargo build --locked --offline --manifest-path "$manifest" -p ic-storage-benchmarks \
    --bin ic-storage-benchmarks --target-dir target/runtime-qualification > "$output/driver-build.log" 2>&1; then
    cat "$output/driver-build.log" >&2; exit 1
fi
shasum -a 256 "$output/owned.wasm" "$output/dfinity.wasm" target/runtime-qualification/debug/ic-storage-benchmarks >> "$output/inputs.sha256"
shasum -a 256 -c "$output/source.sha256" > "$output/source-check.log"
shasum -a 256 -c "$output/inputs.sha256" > "$output/inputs-check.log"
cmp testing/runtime-qualification/Cargo.lock "$output/Cargo.lock"
cmp Cargo.lock "$output/production.lock"
status=0
env -u POCKET_IC_BIN -u IC_TESTKIT_POCKET_IC_URL "$cli" run --directory "$server_directory" --ttl 900 \
    --server-stdout "$output/server.stdout" --server-stderr "$output/server.stderr" \
    -- "$root/target/runtime-qualification/debug/ic-storage-benchmarks" \
    --owned-wasm "$output/owned.wasm" --dfinity-wasm "$output/dfinity.wasm" \
    --output "$output/report" --rows "$rows" --samples "$samples" > "$output/runtime.log" 2>&1 || status=$?
cat "$output/runtime.log"
[[ "$status" == 0 ]] || exit "$status"
shasum -a 256 -c "$output/source.sha256" >> "$output/source-check.log"
shasum -a 256 -c "$output/inputs.sha256" >> "$output/inputs-check.log"
cmp testing/runtime-qualification/Cargo.lock "$output/Cargo.lock"
cmp Cargo.lock "$output/production.lock"
printf 'Comparison summary: %s/report/summary.csv\n' "$output"
