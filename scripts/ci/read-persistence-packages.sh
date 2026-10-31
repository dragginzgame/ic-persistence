#!/usr/bin/env bash
# Read the canonical group order before effects; --all is integration qualification.
set -euo pipefail
[[ $# == 0 || ( $# == 1 && "$1" == --all ) ]] || exit 2
jq -er --arg group "${RELEASE_GROUP:-}" --arg mode "${1:-}" '
  if .schema != 1 then error("unsupported release-group policy") else
    (if $mode == "--all" then [.groups[].members[]]
     else [.groups[] | select(.name == $group) | .members[]] end)
    | if length > 0 and (unique | length) == length and all(.[]; type == "string" and test("^[a-z][a-z0-9-]*$")) then @tsv
      else error("unknown or invalid Persistence release group") end end
' ci/cargo-release-groups.json
