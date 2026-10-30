#!/usr/bin/env bash
# Qualify the exact workspace archives without fetching an unpublished dependency.
set -euo pipefail
[[ $# -ge 1 && $# -le 2 ]] || exit 2
pin="$1"
shift
[[ $# -eq 0 || "$1" == --allow-dirty ]] || exit 2
export RUSTUP_AUTO_INSTALL=0
msrv="$(bash scripts/ci/check-validation-compilers.sh "$pin")"
package_names="$(bash scripts/ci/read-persistence-packages.sh)"
IFS=$'\t' read -r -a packages <<< "$package_names"
source_root="$(pwd -P)"
cargo "+$pin" package --workspace --registry crates-io --locked --offline --no-verify "$@"
metadata="$(cargo "+$pin" metadata --no-deps --locked --offline --format-version 1)"
target="$(printf '%s' "$metadata" | jq -er '.target_directory')"
mkdir -p "$target/package-qualification"
attempt="$(mktemp -d "$target/package-qualification/attempt.XXXXXX")"
echo "Retained package qualification: $attempt"
manifests=()
for package in "${packages[@]}"; do manifests+=("crates/$package/Cargo.toml"); done
shasum -a 256 Cargo.toml Cargo.lock "${manifests[@]}" > "$attempt/source.sha256"
yq -p toml -o json '.' Cargo.lock > "$attempt/selected-lock.json"
extracted=()
for package in "${packages[@]}"; do
    version="$(printf '%s' "$metadata" | jq -er --arg name "$package" '.packages[] | select(.name == $name) | .version')"
    archive="$target/package/$package-$version.crate"
    shasum -a 256 "$archive" >> "$attempt/archives.sha256"
    tar -xzf "$archive" -C "$attempt"
    extracted+=("$attempt/$package-$version")
    jq -nc --arg name "$package" --arg root "$attempt/$package-$version" \
        '{name: $name, root: $root, manifest: ($root + "/Cargo.toml")}' >> "$attempt/packages.ndjson"
done
jq -s '.' "$attempt/packages.ndjson" > "$attempt/packages.json"
# One admitted extracted-path map drives workspace membership and overrides.
# Normalized packaged manifests remain untouched.
mkdir -p "$attempt/consumer/src"
cp testing/package-consumer.rs "$attempt/consumer/src/main.rs"
# Match the public arithmetic types selected by the exact extracted adapter.
metrics="$(jq -er '.[] | select(.name == "ic-metrics-stable") | .root' "$attempt/packages.json")"
metrics_requirement="$(yq -p toml -o yaml -r '.dependencies.ic-metrics.version' "$metrics/Cargo.toml")"
logging="$(jq -er '.[] | select(.name == "ic-logging-stable") | .root' "$attempt/packages.json")"
logging_requirement="$(yq -p toml -o yaml -r '.dependencies.ic-logging.version' "$logging/Cargo.toml")"
# Project every disposable manifest from the same extracted-package observation.
perl "$source_root/scripts/ci/render-package-manifests.pl" "$attempt/packages.json" "$metrics_requirement" "$logging_requirement" "$msrv" "$attempt"
# Seed each disposable graph from the selected lock. Cargo may prune unused
# entries or add the fixture root, but external selections must remain exact.
admit_graph() {
    local manifest="$1" directory="$2"
    cp "$source_root/Cargo.lock" "$directory/Cargo.lock"
    cargo "+$pin" metadata --offline --format-version 1 --manifest-path "$manifest" --config "$attempt/patch.toml" > "$directory/metadata.json"
    yq -p toml -o json '.' "$directory/Cargo.lock" | jq -e -L "$source_root/scripts/ci" \
        --slurpfile selected "$attempt/selected-lock.json" \
        'include "package-graph"; admit_selected_lock($selected[0])' > /dev/null
    jq -e -L "$source_root/scripts/ci" --slurpfile extracted "$attempt/packages.json" \
        --arg fixture "$attempt/consumer/Cargo.toml" \
        'include "package-graph"; admit_extracted_packages($extracted[0]; $fixture)' \
        "$directory/metadata.json" > /dev/null
}
find "${extracted[@]}" -type f -print0 | xargs -0 shasum -a 256 > "$attempt/unpacked.sha256"
admit_graph "$attempt/Cargo.toml" "$attempt"
for package in "${extracted[@]}"; do
    cargo "+$pin" check --locked --offline --manifest-path "$package/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/build"
done
admit_graph "$attempt/consumer/Cargo.toml" "$attempt/consumer"
for compiler in "$pin" "$msrv"; do
    cargo "+$compiler" run --locked --offline --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/consumer-build"
done
cargo "+$pin" check --locked --offline --target wasm32-unknown-unknown --manifest-path "$attempt/consumer/Cargo.toml" --config "$attempt/patch.toml" --target-dir "$attempt/consumer-build"
shasum -a 256 -c "$attempt/archives.sha256"
shasum -a 256 -c "$attempt/unpacked.sha256" > "$attempt/unpacked-check.log"
shasum -a 256 -c "$attempt/source.sha256"
