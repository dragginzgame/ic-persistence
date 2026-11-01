#!/usr/bin/env bash
set -euo pipefail
exec "${SHELLCHECK_BIN:-shellcheck}" --norc --shell=bash ci/release-tools.env ci/runtime-tools.env ci/tool-versions.env scripts/ci/*.sh scripts/dev/*.sh .githooks/pre-commit
