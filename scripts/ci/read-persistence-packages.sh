#!/usr/bin/env bash
# Read the release adapter's one ordered roster before any Cargo/Git effects.
set -euo pipefail
jq -er '
    if type == "array" and length == 3 and
       all(.[]; type == "string" and test("^[a-z][a-z0-9-]*$")) and
       (unique | length) == length
    then @tsv else error("invalid Persistence package roster") end
' crates/ic-memory/examples/repo_tool/packages.json
