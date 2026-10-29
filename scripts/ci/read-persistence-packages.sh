#!/usr/bin/env bash
# Read the canonical coordinated release and publication order before effects.
set -euo pipefail
[[ $# == 0 ]] || exit 2
jq -er '
    if type == "array" and length == 4 and
       all(.[]; type == "string" and test("^[a-z][a-z0-9-]*$")) and
       (unique | length) == length
    then @tsv else error("invalid Persistence package roster") end
' crates/ic-memory/examples/repo_tool/packages.json
