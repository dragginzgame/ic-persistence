#!/usr/bin/env bash
set -euo pipefail
[[ $# -eq 1 ]] || exit 2
case "$1" in publish) mode=publish ;; publish-check) mode=check ;; *) exit 2 ;; esac
bash scripts/ci/check-release-source.sh
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
commit="$(git rev-parse HEAD)"
bash scripts/ci/check-release-tag.sh "$commit" "$version"
make --no-print-directory clippy
# The shared owner retains exact dispatch arguments and interrupted-attempt evidence.
mkdir -p target/publication
attempt="$(mktemp -d target/publication/attempt.XXXXXX)"
rmdir "$attempt"
bash scripts/ci/run-cargo-publication.sh "$mode" "$attempt" "$commit" crates-io \
    --package "ic-storage@$version" --all-features
