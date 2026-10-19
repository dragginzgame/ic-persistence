#!/usr/bin/env bash
set -euo pipefail
[[ $# -eq 0 || ( $# -eq 1 && "$1" == --check ) ]] || exit 2
# Release-only host tool; common development setup does not install it.
exec bash scripts/dev/install-rust-tools.sh --consumer "$PWD" \
    --package ic-build --version 0.18.0 --bin ic-build-release-metadata \
    --profile debug "$@"
