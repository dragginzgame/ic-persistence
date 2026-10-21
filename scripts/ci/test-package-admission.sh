#!/usr/bin/env bash
# Exercise the real archive admission before any compilation, with bad observations.
set -euo pipefail
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-persistence-package-admission.XXXXXX")"
echo "Retained archive-admission fixture: $fixture"
mkdir -p "$fixture/bin"
REAL_CARGO="$(command -v cargo)"
REAL_YQ="$(command -v yq)"
export REAL_CARGO REAL_YQ
pin="$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)"
# Packaging is setup, not the behavior under fault injection. Prepare it once;
# both rejection cases still run the real extraction and graph admission path.
"$REAL_CARGO" "+$pin" package --workspace --registry crates-io --locked --offline --no-verify --allow-dirty > "$fixture/package.log" 2>&1
metadata="$("$REAL_CARGO" "+$pin" metadata --no-deps --locked --offline --format-version 1)"
target="$(printf '%s' "$metadata" | jq -er '.target_directory')"
PREPARED_ARCHIVES="$fixture/archives.sha256"
PACKAGE_TOOLCHAIN="$pin"
export PREPARED_ARCHIVES PACKAGE_TOOLCHAIN
for package in ic-memory ic-storage; do
    version="$(printf '%s' "$metadata" | jq -er --arg name "$package" '.packages[] | select(.name == $name) | .version')"
    shasum -a 256 "$target/package/$package-$version.crate" >> "$PREPARED_ARCHIVES"
done
cat > "$fixture/bin/yq" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$REJECT_MODE" == drift && "${*: -1}" == */package-qualification/attempt.*/Cargo.lock ]]; then
    "$REAL_YQ" "$@" | jq '(.package[] | select(.source != null) | .version) = "999.0.0"'
else
    exec "$REAL_YQ" "$@"
fi
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == *' package '* ]]; then
    [[ "$*" == "+$PACKAGE_TOOLCHAIN package --workspace --registry crates-io --locked --offline --no-verify --allow-dirty" ]] || exit 98
    shasum -a 256 -c "$PREPARED_ARCHIVES" > /dev/null
    exit 0
fi
if [[ "$*" == *' check '* || "$*" == *' run '* ]]; then
    echo 'unexpected compilation after invalid admission' >&2
    exit 99
fi
if [[ "$REJECT_MODE" == duplicate && "$*" == *' metadata '* && "$*" == *'/package-qualification/attempt.'* ]]; then
    "$REAL_CARGO" "$@" | jq '.packages += [.packages[] | select(.name == "ic-memory")]'
else
    exec "$REAL_CARGO" "$@"
fi
STUB
chmod +x "$fixture/bin/cargo" "$fixture/bin/yq"
for mode in drift duplicate; do
    if REJECT_MODE="$mode" PATH="$fixture/bin:$PATH" bash scripts/ci/package-workspace.sh "$pin" --allow-dirty > "$fixture/$mode.log" 2>&1; then
        echo "Incorrectly admitted $mode; see $fixture/$mode.log" >&2
        exit 1
    fi
    if rg -q 'unexpected compilation' "$fixture/$mode.log"; then
        cat "$fixture/$mode.log" >&2
        exit 1
    fi
    case "$mode" in
        drift) expected='archive dependency selection drift' ;;
        duplicate) expected='archive graph must use exactly the extracted Memory and Storage packages' ;;
    esac
    rg -q "$expected" "$fixture/$mode.log" || { cat "$fixture/$mode.log" >&2; exit 1; }
    echo "Rejected $mode before compilation"
done
shasum -a 256 -c "$PREPARED_ARCHIVES"
