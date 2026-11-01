#!/usr/bin/env bash
set -euo pipefail
files="$(printf '%s\n' Makefile ci/*.env scripts/ci/*.sh scripts/dev/*.sh .githooks/pre-commit \
    "${SHELLCHECK_BIN:-$(command -v shellcheck)}" "$(command -v jq)" | jq -Rsc 'split("\n")[:-1]')"
jq -cnS --argjson files "$files" '{schema:1,files:$files,configuration:{shell:"bash",config:"none",scope:"consumer-tooling"}}'
