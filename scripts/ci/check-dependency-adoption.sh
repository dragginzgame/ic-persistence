#!/usr/bin/env bash
# Read-only destination admission before release planning or artifact work.
set -euo pipefail
[[ $# == 1 && -n "$1" ]] || { echo 'usage: check-dependency-adoption.sh TOOLCHAIN' >&2; exit 2; }
export RUSTUP_TOOLCHAIN="$1" RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true
shared=scripts/ci
evidence="$(mktemp -d "${TMPDIR:-/tmp}/storage-adoption.XXXXXX")"
printf 'Dependency adoption evidence: %s\n' "$evidence"
inputs=(Cargo.toml Cargo.lock crates/*/Cargo.toml testing/runtime-qualification/Cargo.toml \
    testing/runtime-qualification/Cargo.lock testing/runtime-qualification/crates/*/Cargo.toml)
checkpoint() {
    perl "$shared/validation-input-checkpoint.pl" "$PWD" "$1" "$2" "$shared/verify-file-checksum.sh" "${inputs[@]}"
}
checkpoint '' "$evidence/before.json"
host="$(rustc -vV | sed -n 's/^host: //p')"
[[ "$host" =~ ^[A-Za-z0-9_.-]+$ ]] || { echo 'invalid native compiler target' >&2; exit 1; }
# Observe every independent lock before policy or compilation. Never unlock,
# update a version, refresh artifacts or authenticate plausible failed stdout.
cargo metadata --locked --offline --format-version 1 --manifest-path Cargo.toml > "$evidence/library.json"
cargo metadata --locked --offline --format-version 1 --manifest-path testing/runtime-qualification/Cargo.toml > "$evidence/runtime.json"
perl "$shared/check-cargo-graph.pl" --consumer "$PWD" \
    --package ic-storage --package ic-metrics-storage --package ic-logging-storage \
    --target wasm32-unknown-unknown --edges normal --explain \
    --forbid pocket-ic --forbid-feature ic-storage/test-support \
    --require ic-storage=1 --require ic-platform=1 --require ic-wire=1
# The native qualifier exchanges Platform/Wire types and opaque Wasm bytes;
# it deliberately has no normal dependency on the local Memory/Storage crates.
perl "$shared/check-cargo-graph.pl" --consumer "$PWD/testing/runtime-qualification" \
    --package ic-storage-runtime-qualification --target "$host" --edges normal --explain \
    --require ic-platform=1 --require ic-wire=1
checkpoint "$evidence/before.json" "$evidence/graphs.json"
cargo clippy --locked --offline --manifest-path Cargo.toml --target wasm32-unknown-unknown \
    --package ic-storage --package ic-metrics-storage --package ic-logging-storage --lib -- -D warnings
cargo clippy --locked --offline --manifest-path testing/runtime-qualification/Cargo.toml \
    --package ic-storage-runtime-qualification --target "$host" --all-targets -- -D warnings
checkpoint "$evidence/before.json" "$evidence/final.json"
echo 'Destination graphs and selected consumer compilation admitted; artifact/runtime/publication qualification remains separate.'
