#!/usr/bin/env bash
# Consumer roster only; Shared admits formatter prerequisites before dispatch.
set -euo pipefail
[[ $# == 2 ]] || exit 2
compiler="$1"
mode="$2"
sort_flags=() rustfmt_flags=() cargo_fmt_flags=()
case "$mode" in
    write) ;;
    check) sort_flags=(--check); rustfmt_flags=(--check); cargo_fmt_flags=(-- --check) ;;
    *) exit 2 ;;
esac
for workspace in . testing/runtime-qualification; do
    cargo "+$compiler" sort --workspace --grouped ${sort_flags[@]+"${sort_flags[@]}"} "$workspace"
done
for workspace in . testing/runtime-qualification; do
    cargo "+$compiler" fmt --manifest-path "$workspace/Cargo.toml" --all ${cargo_fmt_flags[@]+"${cargo_fmt_flags[@]}"}
done
rustfmt "+$compiler" --edition 2024 ${rustfmt_flags[@]+"${rustfmt_flags[@]}"} testing/package-consumer.rs
