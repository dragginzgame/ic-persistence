#!/usr/bin/env bash
# Read the release adapter's publication or complete workspace roster.
set -euo pipefail
rosters=(crates/ic-memory/examples/repo_tool/packages.json)
case "$*" in
    '') [[ $# == 0 ]] || exit 2 ;;
    --workspace) [[ $# == 1 ]] || exit 2; rosters+=(crates/ic-memory/examples/repo_tool/unpublished-packages.json) ;;
    *) echo 'usage: read-persistence-packages.sh [--workspace]' >&2; exit 2 ;;
esac
jq -ers '
    def roster($count): type == "array" and length == $count and
        all(.[]; type == "string" and test("^[a-z][a-z0-9-]*$"));
    if (length == 1 or length == 2) and (.[0] | roster(3)) and
       (length == 1 or (.[1] | roster(1))) then
        add | if (unique | length) == length then @tsv
              else error("duplicate Persistence package") end
    else error("invalid Persistence package roster") end
' "${rosters[@]}"
