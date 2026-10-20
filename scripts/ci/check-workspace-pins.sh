#!/usr/bin/env bash
# Preserve published versions until the explicitly authorized coordinated release.
set -euo pipefail
storage="$(yq -p toml -o json '.' crates/ic-storage/Cargo.toml | jq -c '.package.version')"
if [[ "$storage" != '"0.5.9"' ]]; then
    exec bash scripts/ci/check-dependency-pins.sh --cargo-inheritance
fi
[[ "$(yq -p toml -o yaml -r '.workspace.package.version' Cargo.toml)" == 0.49.4 ]] || {
    echo 'explicit Storage version is allowed only before the 0.50.0 consolidation release' >&2
    exit 1
}
bash scripts/ci/check-dependency-pins.sh
# Reuse the canonical inheritance implementation; exempt only the exact approved
# Storage version finding, not its dependencies or any other package metadata.
for workspace in . testing/runtime-qualification; do
    metadata="$(cargo metadata --locked --offline --no-deps --format-version 1 --manifest-path "$workspace/Cargo.toml")"
    root_json="$(yq -p toml -o json '.' "$workspace/Cargo.toml")"
    members="$(printf '%s' "$metadata" | jq -er '.packages[].manifest_path')"
    while IFS= read -r manifest; do
        relative="${manifest#"$PWD/"}"
        yq -p toml -o json '.' "$manifest" | jq -e -L scripts/ci \
            --arg file "$relative" --argjson root "$root_json" '
            include "dependency-pins";
            [{file: $file, data: .} | inheritance_checks($root; true) |
                select(.file != "crates/ic-storage/Cargo.toml" or .rule != "cargo-inheritance" or
                    .subject != "package.version" or .value != "\"0.5.9\"")]
            | if length == 0 then true else error(tostring) end' > /dev/null
    done <<< "$members"
done
echo 'Cargo inheritance passed with the documented pre-release Storage version exception.'
