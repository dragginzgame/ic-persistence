#!/usr/bin/env bash
# Offline admission only: setup owns compiler installation.
set -euo pipefail
[[ $# == 1 && "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || exit 2
msrv="$(yq -p toml -o yaml -r '.workspace.package.rust-version' Cargo.toml)"
[[ "$msrv" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'invalid declared MSRV' >&2; exit 1; }
for compiler in "$1" "$msrv"; do
    rustup run "$compiler" rustc --version || {
        echo "Prepare required compiler first: rustup toolchain install --profile minimal $compiler" >&2
        exit 1
    }
done
