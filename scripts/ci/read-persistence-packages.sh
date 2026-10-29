#!/usr/bin/env bash
# Read the release adapter's one ordered roster before any Cargo/Git effects.
set -euo pipefail
rosters=(crates/ic-memory/examples/repo_tool/packages.json)
case "${1:-}" in
    '') [[ $# == 0 ]] || exit 2 ;;
    --workspace) [[ $# == 1 ]] || exit 2; rosters+=(crates/ic-memory/examples/repo_tool/unpublished-packages.json) ;;
    *) exit 2 ;;
esac
jq -ser '
    if (.[0] | type == "array" and length == 3) and
       (length == 1 or (.[1] | type == "array" and length == 1))
    then add else error("invalid Persistence roster size") end
    | if type == "array" and
       all(.[]; type == "string" and test("^[a-z][a-z0-9-]*$")) and
       (unique | length) == length
    then @tsv else error("invalid Persistence package roster") end
' "${rosters[@]}"
