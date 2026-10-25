#!/usr/bin/env bash
# Actual offline Cargo and the selected editor; no fixture Git/release effects.
set -euo pipefail
package_names="$(bash scripts/ci/read-persistence-packages.sh)"
IFS=$'\t' read -r -a packages <<< "$package_names"
unset RELEASE_CACHE_PREPARE MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
root="$(pwd -P)"
pin="$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)"
# shellcheck source=ci/release-tools.env
source ci/release-tools.env
editor="$(RUSTUP_TOOLCHAIN="$pin" bash scripts/dev/install-rust-tools.sh --consumer "$root" \
    --package ic-build --version "$IC_BUILD_RELEASE_METADATA_VERSION" --bin ic-build-release-metadata --profile debug --features cli --check)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-persistence-bootstrap.XXXXXX")"
echo "Retained coordinated bootstrap fixture: $fixture"
cp Cargo.toml Cargo.lock LICENSE rust-toolchain.toml "$fixture/"
cp -R crates "$fixture/"
mkdir -p "$fixture/scripts/dev" "$fixture/scripts/ci" "$fixture/ci"
cp scripts/dev/run-repo-tool.sh "$fixture/scripts/dev/"
cp scripts/ci/read-persistence-packages.sh scripts/ci/read-cargo-workspace-version.sh scripts/ci/validate-release-inputs.pl scripts/ci/StableVersion.pm "$fixture/scripts/ci/"
cp ci/release-tools.env "$fixture/ci/"
cat > "$fixture/scripts/dev/install-rust-tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "${*: -1}" == --check ]] || exit 1
printf '%s\n' "$BOOTSTRAP_QUALIFIED_EDITOR"
STUB
export BOOTSTRAP_QUALIFIED_EDITOR="$editor"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR="$fixture/target"
cd "$fixture"
previous="$(yq -p toml -o yaml -r '.workspace.package.version' Cargo.toml)"
next="$(bash "$root/scripts/ci/next-release-version.sh" "$previous" minor)"
cp Cargo.toml original.toml
cp Cargo.lock original.lock
cargo "+$pin" metadata --locked --offline --no-deps --format-version 1 > metadata.json
yq -p toml -o json '.' Cargo.toml | jq --slurpfile metadata metadata.json '{manifest: ., members: ($metadata[0].packages | map({name, manifest_path}))}' > context.json
"$editor" --max-input-bytes "$CARGO_RELEASE_METADATA_MAX_BYTES" manifest Cargo.toml "$previous" "$next" context.json > candidate.toml
# Only these disposable fixture inputs are edited; production release metadata
# remains unchanged. All members inherit the coordinated workspace version.
cp Cargo.lock candidate.lock
for package in "${packages[@]}"; do
    package_previous="$(yq -p toml -o json '.' Cargo.lock | jq -er --arg package "$package" '.package[] | select(.name == $package and .source == null) | .version')"
    "$editor" --max-input-bytes "$CARGO_RELEASE_METADATA_MAX_BYTES" lock candidate.lock "$package_previous" "$next" "$package" > candidate-next.lock
    mv candidate-next.lock candidate.lock
done
for state in lock-first manifest-first; do
    cp original.toml Cargo.toml
    cp original.lock Cargo.lock
    expected="$previous"
    case "$state" in
        lock-first) cp candidate.lock Cargo.lock ;;
        manifest-first) cp candidate.toml Cargo.toml; expected="$next" ;;
    esac
    shasum -a 256 Cargo.toml Cargo.lock crates/ic-memory/Cargo.toml crates/ic-storage/Cargo.toml crates/ic-metrics-stable/Cargo.toml > "$state.inputs.sha256"
    bash scripts/dev/run-repo-tool.sh "$pin" version > "$state.stdout" 2> "$state.stderr"
    [[ "$(cat "$state.stdout")" == "$expected" ]] || exit 1
    shasum -a 256 -c "$state.inputs.sha256"
done
echo 'Coordinated bootstrap admits both interrupted metadata states without modifying tracked inputs.'
