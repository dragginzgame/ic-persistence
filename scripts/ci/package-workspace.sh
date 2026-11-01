#!/usr/bin/env bash
# Qualify the exact workspace archives without fetching an unpublished dependency.
set -euo pipefail
[[ $# -ge 1 && $# -le 2 ]] || exit 2
pin="$1"
shift
[[ $# -eq 0 || "$1" == --allow-dirty ]] || exit 2
export RUSTUP_AUTO_INSTALL=0
msrv="$(bash scripts/ci/check-validation-compilers.sh "$pin")"
bash scripts/ci/read-persistence-packages.sh --all > /dev/null
cargo "+$pin" package --workspace --registry crates-io --locked --offline --no-verify "$@"
# The integration bundle may synthesize local supplier checksums. Repackage the
# release group alone so already-published suppliers use their registry identity;
# coordinated unpublished members still share one Cargo packaging invocation.
if [[ -n "${RELEASE_GROUP:-}" ]]; then
    selected_names="$(bash scripts/ci/read-persistence-packages.sh)"
    IFS=$'\t' read -r -a selected_packages <<< "$selected_names"
    selection=()
    for package in "${selected_packages[@]}"; do selection+=(--package "$package"); done
    cargo "+$pin" package "${selection[@]}" --registry crates-io --locked --offline --no-verify "$@"
fi
metadata="$(cargo "+$pin" metadata --no-deps --locked --offline --format-version 1)"
target="$(printf '%s' "$metadata" | jq -er '.target_directory')"
mkdir -p "$target/package-qualification"
attempt="$(mktemp -d "$target/package-qualification/attempt.XXXXXX")"
echo "Retained package qualification: $attempt"
# Build owns extraction, frozen package admission and final archive/source custody.
# The native consumer retains its explicit library/native/MSRV/Wasm check roster.
bash scripts/dev/run-repo-tool.sh "$pin" qualify-archives "$attempt" "$msrv"
