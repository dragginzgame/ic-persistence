#!/usr/bin/env bash
set -euo pipefail
# Qualify this consumer's actual preflight with substituted Git/Cargo/tool effects.
root="$(cd -P "${BASH_SOURCE[0]%/*}/../.." && pwd)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-storage-preflight.XXXXXX")"
fixture="$(cd -P "$fixture" && pwd)"
finish() {
    local status=$?
    if [[ "$status" == 0 ]]; then rm -rf -- "$fixture"
    else printf 'Preflight fixture and logs retained: %s\n' "$fixture" >&2; fi
}
trap finish EXIT
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture" \
    scripts/release/adapter.sh scripts/release/editor.sh \
    scripts/ci/check-release-delivery.sh scripts/ci/check-release-source.sh \
    scripts/ci/read-cargo-workspace-version.sh scripts/ci/finalize-release-changelog.awk \
    scripts/ci/cargo-release-preflight.sh ci/tool-versions.env ci/release-delivery.txt
mkdir -p "$fixture/bin" "$fixture/scripts/dev"
printf '[workspace.package]\nversion = "0.1.0"\n' > "$fixture/Cargo.toml"
printf '# Changelog\n\n## [0.1.1]\n\n- Fixture change.\n\n## [0.1.0] - 2026-01-01\n' > "$fixture/CHANGELOG.md"
printf 'unchanged fixture lock\n' > "$fixture/Cargo.lock"
for path in Cargo.toml Cargo.lock CHANGELOG.md; do
    cp "$fixture/$path" "$fixture/$path.before"
done

cat > "$fixture/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    'rev-parse --show-prefix'|'status --porcelain=v1 -z --untracked-files=all') ;;
    *) echo 'unexpected Git effect' >&2; exit 2 ;;
esac
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ -z "${IC_STORAGE_RELEASE_PREPARE+x}" ]] || exit 2
case "$*" in
    'locate-project --workspace --message-format plain --manifest-path Cargo.toml') ;;
    'fetch --locked'|'fetch --locked --offline') printf '%s\n' "$*" >> "$PREFLIGHT_EVENTS" ;;
    *) echo 'unexpected Cargo effect' >&2; exit 2 ;;
esac
STUB
cat > "$fixture/bin/make" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 4 && "$1" == --no-print-directory && "$2" == -C && "$3" == "$PWD" ]] || exit 2
printf '%s\n' "$4" >> "$PREFLIGHT_EVENTS"
case "$4" in
    install-tools|tools-check) ;;
    install-release-tools) bash scripts/release/editor.sh ;;
    release-tools-check) bash scripts/release/editor.sh --check > /dev/null ;;
    *) echo 'unexpected Make target' >&2; exit 2 ;;
esac
STUB
# The real editor selector must use this same pinned package/profile for setup/check.
cat > "$fixture/scripts/dev/install-rust-tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 10 || ( $# == 11 && "${11}" == --check ) ]] || exit 2
[[ "$1" == --consumer && "$2" == "$PWD" && "$3" == --package && "$4" == ic-build &&
   "$5" == --version && "$6" == 0.18.0 && "$7" == --bin && "$8" == ic-build-release-metadata &&
   "$9" == --profile && "${10}" == debug ]] || exit 2
if [[ $# == 11 ]]; then
    [[ -f "$PREFLIGHT_EDITOR" && "$(cat "$PREFLIGHT_EDITOR")" == "$PREFLIGHT_COMPILER" ]] || exit 37
else
    [[ "${PREFLIGHT_INSTALL_FAIL:-0}" == 0 ]] || exit 73
    printf '%s\n' "$PREFLIGHT_COMPILER" > "$PREFLIGHT_EDITOR"
fi
STUB
chmod +x "$fixture/bin/git" "$fixture/bin/cargo" "$fixture/bin/make"
export PATH="$fixture/bin:$PATH"
export PREFLIGHT_EVENTS="$fixture/events" PREFLIGHT_EDITOR="$fixture/editor-state"
export PREFLIGHT_COMPILER=compiler-a RELEASE_MAKE="$fixture/bin/make"
export RELEASE_PREVIOUS=0.1.0 RELEASE_VERSION=0.1.1 RELEASE_DATE=2026-01-02
unset RELEASE_COMMIT
cd "$fixture"

run_preflight() {
    : > "$PREFLIGHT_EVENTS"
    IC_STORAGE_RELEASE_PREPARE="$1" bash scripts/release/adapter.sh preflight > "$2.log" 2>&1
}
expect_events() {
    printf '%s\n' "$@" > expected
    cmp expected "$PREFLIGHT_EVENTS"
    for path in Cargo.toml Cargo.lock CHANGELOG.md; do cmp "$path.before" "$path"; done
}

# A fresh release supplies the editor and checks the prepared installation.
run_preflight 1 fresh
expect_events 'fetch --locked' install-tools install-release-tools tools-check release-tools-check
[[ "$(cat "$PREFLIGHT_EDITOR")" == compiler-a ]]

# Standalone preflight admits a prepared editor without installation or network.
run_preflight 0 prepared
expect_events 'fetch --locked --offline' tools-check release-tools-check

# A missing or compiler-invalidated installation refuses in check-only mode.
rm "$PREFLIGHT_EDITOR"
if run_preflight 0 missing; then echo 'missing editor was accepted' >&2; exit 1; fi
expect_events 'fetch --locked --offline' tools-check release-tools-check
[[ ! -e "$PREFLIGHT_EDITOR" ]]
printf 'compiler-a\n' > "$PREFLIGHT_EDITOR"
export PREFLIGHT_COMPILER=compiler-b
if run_preflight 0 stale; then echo 'stale editor was accepted' >&2; exit 1; fi
expect_events 'fetch --locked --offline' tools-check release-tools-check
[[ "$(cat "$PREFLIGHT_EDITOR")" == compiler-a ]]

# Authorized preparation repairs the selection after a compiler change.
run_preflight 1 changed-compiler
expect_events 'fetch --locked' install-tools install-release-tools tools-check release-tools-check
[[ "$(cat "$PREFLIGHT_EDITOR")" == compiler-b ]]

# Failed installation stops before later checks, preserving metadata and receipt.
export PREFLIGHT_INSTALL_FAIL=1
if run_preflight 1 refused; then echo 'failed setup was accepted' >&2; exit 1; fi
expect_events 'fetch --locked' install-tools install-release-tools
[[ "$(cat "$PREFLIGHT_EDITOR")" == compiler-b ]]
echo 'IC Storage release preflight passed six consumer scenarios (substituted tool effects).'
