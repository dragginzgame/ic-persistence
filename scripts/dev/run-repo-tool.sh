#!/usr/bin/env bash
# Keep the Rust adapter reachable between atomic manifest/lockfile replacements.
set -euo pipefail
export RUSTUP_AUTO_INSTALL=0
# Cargo's --offline flags scope helper compilation/discovery. Preserve the
# caller's network setting for commands the helper dispatches, such as publish.
if [[ $# -lt 2 ]]; then
    echo 'usage: run-repo-tool.sh <toolchain> <command> [arguments]' >&2
    exit 2
fi
toolchain="$1"
shift
check_dependency_cache() {
    local status
    local fetch=(fetch --locked --offline)
    # Only the documented release entry points select network preparation.
    # Cargo still honours explicit offline environment/configuration settings.
    if [[ "${RELEASE_CACHE_PREPARE:-0}" == 1 ]]; then
        fetch=(fetch --locked)
    fi
    if cargo "+$toolchain" "${fetch[@]}" "$@"; then
        # The compiled adapter and its validation children stay offline.
        unset RELEASE_CACHE_PREPARE
        return
    else
        status=$?
    fi
    echo 'Dependency preparation failed. Run make fetch-dependencies to populate the selected cache; resolve any reported lockfile or network errors first. Explicit offline settings remain in effect.' >&2
    exit "$status"
}
case "$1" in
    version|release-*|qualify-release|publish)
        [[ -n "${RELEASE_GROUP:-}" ]] || { echo 'Select RELEASE_GROUP explicitly.' >&2; exit 2; }
        jq -e --arg group "$RELEASE_GROUP" 'any(.groups[]; .name == $group)' ci/cargo-release-groups.json > /dev/null || exit 2
        ;;
esac
# Cargo admits TOML syntax (including duplicate keys) without resolving locks.
RUSTUP_TOOLCHAIN="$toolchain" CARGO_NET_OFFLINE=true cargo locate-project --workspace --message-format plain --manifest-path Cargo.toml > /dev/null
# Admit the virtual catalog before cache/helper effects, without a master version.
catalog_json="$("${YQ:-yq}" -p toml -o json '.' Cargo.toml)"
printf '%s\n' "$catalog_json" | jq -e '
  if has("package") or (.workspace | type) != "object" or .workspace.package.version != null
  then error("independent groups require a virtual catalog without workspace.package.version") else true end' > /dev/null
bash scripts/ci/read-persistence-packages.sh --all > /dev/null
lock_json="$("${YQ:-yq}" -p toml -o json '.' Cargo.lock)" || exit 1
# One complete observation drives coherence and any scratch projection.
package_versions="$(printf '%s\n' "$lock_json" | jq -er --slurpfile policy ci/cargo-release-groups.json '
  . as $lock | [$policy[0].groups[] | . as $group |
    [.members[] as $name | [$lock.package[] | select(.name == $name and .source == null)] |
     if length == 1 and (.[0].version | type) == "string" and (.[0].version | explode | all(. >= 32)) and
       (.[0].version | test("^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$"))
     then {name:$name, version:.[0].version, style:$group.version}
     else error("invalid local lock identity") end] |
    if ([.[].version] | unique | length) == 1 then .[]
    else error("release group lock versions differ") end] |
    .[] | [.name,.version,.style] | @tsv')" || exit 1
coherent=true
declared_versions=()
locked_versions=()
while IFS=$'\t' read -r package locked style; do
    declared="$("${YQ:-yq}" -p toml -o json '.' "crates/$package/Cargo.toml" | jq -er --arg style "$style" '.package.version | if $style == "package" and type == "string" then . else error("invalid group version source") end')"
    declared_versions+=("$declared")
    locked_versions+=("$locked")
    [[ "$locked" == "$declared" ]] || coherent=false
done <<< "$package_versions"
if [[ "$coherent" == true ]]; then
    check_dependency_cache
    exec cargo "+$toolchain" run --locked --offline --quiet --example repo-tool -- "$@"
fi
# --no-deps metadata does not resolve or update the mismatched lock. Respect
# Cargo's selected target directory, including the consumer's configuration.
# Preserve path bytes, removing only the parser's output record terminator.
# The sentinel is appended only after the complete pipeline succeeds.
target="$(cargo "+$toolchain" metadata --locked --offline --no-deps --format-version 1 |
    "${YQ:-yq}" -p=json -r '.target_directory' && printf '.')"
target="${target%$'\n.'}"
[[ "$target" == /* && "$target" != *$'\n'* && "$target" != *$'\r'* ]] || { echo 'invalid bootstrap target directory' >&2; exit 1; }
mkdir -p "$target/repo-tool-bootstrap"
scratch="$(mktemp -d "$target/repo-tool-bootstrap/attempt.XXXXXX")"
echo "Retained repo-tool bootstrap metadata: $scratch" >&2
cp Cargo.lock LICENSE Cargo.toml "$scratch/"
cp -R crates "$scratch/"
# shellcheck source=ci/release-tools.env
source ci/release-tools.env
editor_admission=(--check)
if [[ "${RELEASE_CACHE_PREPARE:-0}" == 1 ]]; then editor_admission=(); fi
editor="$(RUSTUP_TOOLCHAIN="$toolchain" bash scripts/dev/install-rust-tools.sh \
    --consumer "$PWD" --package ic-build --version "$IC_BUILD_RELEASE_METADATA_VERSION" \
    --bin ic-build-release-metadata --profile debug --features cli ${editor_admission[@]+"${editor_admission[@]}"})"
printf '%s\n' "$lock_json" > "$scratch/lock.json"
printf '%s\n' "$catalog_json" | jq --slurpfile lock "$scratch/lock.json" '
    {version:"unchanged", dependencies:[.workspace.dependencies | to_entries[] |
      select((.value|type) == "object" and .value.path != null and .value.version != null) |
      . as $d | ($d.value.package // $d.key) as $package |
      $lock[0].package[] | select(.name == $package and .source == null) |
      {alias:$d.key, previous:$d.value.version, next:.version}]}' > "$scratch/root-edit.json"
# The root is a virtual dependency catalog, with no version literal.
# Unchanged mode still validates the edit versions; the values select no root version.
"$editor" --max-input-bytes "$CARGO_RELEASE_METADATA_MAX_BYTES" group-manifest Cargo.toml \
    0.0.0 0.0.0 "$scratch/root-edit.json" > "$scratch/Cargo.toml"
printf '{"version":"package","dependencies":[]}\n' > "$scratch/member-edit.json"
index=0
while IFS=$'\t' read -r package locked style; do
    manifest="crates/$package/Cargo.toml"
    "$editor" --max-input-bytes "$CARGO_RELEASE_METADATA_MAX_BYTES" group-manifest "$manifest" \
        "${declared_versions[$index]}" "${locked_versions[$index]}" "$scratch/member-edit.json" > "$scratch/$manifest"
    index=$((index + 1))
done <<< "$package_versions"
# Compilation metadata is coherent only in scratch. The adapter still observes
# the real checkout and must admit its saved source, group and receipts.
check_dependency_cache --manifest-path "$scratch/Cargo.toml"
exec cargo "+$toolchain" run --locked --offline --quiet \
    --manifest-path "$scratch/Cargo.toml" --target-dir "$target/repo-tool-bootstrap/build" \
    --example repo-tool -- "$@"
