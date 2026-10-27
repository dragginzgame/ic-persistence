#!/usr/bin/env bash
# Pure archive graph policy over bounded test observations; no Cargo dispatch.
set -euo pipefail
source scripts/ci/fixture-lifecycle.sh
fixture="$(fixture_create "${TMPDIR:-/tmp}/ic-persistence-package-graph.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Package graph fixture retained"' EXIT
# Pure observations cover graph policy without Cargo/compiler/metadata stubs.
jq -n '{package: [{name: "dependency", version: "1.0.0", source: "registry+fixture", checksum: "digest"}]}' > "$fixture/selected.json"
jq -n '[{name: "ic-memory", manifest: "/exact/ic-memory/Cargo.toml"}]' > "$fixture/extracted.json"
jq -n '{packages: [
    {name: "ic-memory", source: null, manifest_path: "/exact/ic-memory/Cargo.toml"},
    {name: "dependency", source: "registry+fixture", manifest_path: "/cache/dependency/Cargo.toml"}
]}' > "$fixture/graph.json"
module_directory="$PWD/scripts/ci"
jq -e -L "$module_directory" --slurpfile selected "$fixture/selected.json" \
    'include "package-graph"; admit_selected_lock($selected[0])' "$fixture/selected.json" > /dev/null
jq -e -L "$module_directory" --slurpfile extracted "$fixture/extracted.json" \
    'include "package-graph"; admit_extracted_packages($extracted[0]; "/consumer/Cargo.toml")' \
    "$fixture/graph.json" > /dev/null
# Pruned external locks and the explicit consumer root remain admissible.
printf '{"package": []}\n' | jq -e -L "$module_directory" --slurpfile selected "$fixture/selected.json" \
    'include "package-graph"; admit_selected_lock($selected[0])' > /dev/null
jq '.packages += [{name: "consumer", source: null, manifest_path: "/consumer/Cargo.toml"}]' \
    "$fixture/graph.json" | jq -e -L "$module_directory" --slurpfile extracted "$fixture/extracted.json" \
    'include "package-graph"; admit_extracted_packages($extracted[0]; "/consumer/Cargo.toml")' > /dev/null
for mode in name version source checksum; do
    jq --arg field "$mode" '(.package[0][$field]) = "changed"' "$fixture/selected.json" > "$fixture/changed.json"
    if jq -e -L "$module_directory" --slurpfile selected "$fixture/selected.json" \
        'include "package-graph"; admit_selected_lock($selected[0])' "$fixture/changed.json" \
        > "$fixture/pure-$mode.log" 2>&1; then
        echo "Incorrectly admitted locked $mode drift" >&2; exit 1
    fi
    rg -q 'archive dependency selection drift' "$fixture/pure-$mode.log"
done
for mode in missing duplicate substituted registry-source local-extra; do
    jq --arg mode "$mode" '
        if $mode == "missing" then .packages |= map(select(.name != "ic-memory"))
        elif $mode == "duplicate" then .packages += [.packages[0]]
        elif $mode == "substituted" then .packages[0].manifest_path = "/substituted/Cargo.toml"
        elif $mode == "registry-source" then .packages[0].source = "registry+fixture"
        else .packages += [{name: "foreign-local", source: null, manifest_path: "/foreign/Cargo.toml"}] end
    ' "$fixture/graph.json" > "$fixture/changed.json"
    if jq -e -L "$module_directory" --slurpfile extracted "$fixture/extracted.json" \
        'include "package-graph"; admit_extracted_packages($extracted[0]; "/consumer/Cargo.toml")' \
        "$fixture/changed.json" > "$fixture/pure-$mode.log" 2>&1; then
        echo "Incorrectly admitted $mode graph" >&2; exit 1
    fi
    rg -q 'archive graph must use exactly the extracted Persistence packages' "$fixture/pure-$mode.log"
done
echo 'Pure package graph admission checks passed'
fixture_complete=true
