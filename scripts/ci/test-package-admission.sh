#!/usr/bin/env bash
# Exercise the real archive admission before any compilation, with bad observations.
set -euo pipefail
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-persistence-package-admission.XXXXXX")"
echo "Retained archive-admission fixture: $fixture"
mkdir -p "$fixture/bin"
REAL_CARGO="$(command -v cargo)"
REAL_YQ="$(command -v yq)"
REAL_RUSTUP="$(command -v rustup)"
export REAL_CARGO REAL_YQ REAL_RUSTUP
pin="$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)"
# Packaging is setup, not the behavior under fault injection. Prepare it once;
# both rejection cases still run the real extraction and graph admission path.
"$REAL_CARGO" "+$pin" package --workspace --registry crates-io --locked --offline --no-verify --allow-dirty > "$fixture/package.log" 2>&1
metadata="$("$REAL_CARGO" "+$pin" metadata --no-deps --locked --offline --format-version 1)"
target="$(printf '%s' "$metadata" | jq -er '.target_directory')"
PREPARED_ARCHIVES="$fixture/archives.sha256"
PACKAGE_TOOLCHAIN="$pin"
export PREPARED_ARCHIVES PACKAGE_TOOLCHAIN
for package in ic-memory ic-storage ic-metrics-stable; do
    version="$(printf '%s' "$metadata" | jq -er --arg name "$package" '.packages[] | select(.name == $name) | .version')"
    shasum -a 256 "$target/package/$package-$version.crate" >> "$PREPARED_ARCHIVES"
done
cat > "$fixture/bin/yq" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$REJECT_MODE" == selected-msrv && "$*" == '-p toml -o yaml -r .workspace.package.rust-version Cargo.toml' ]]; then
    printf '%s\n' "$FIXTURE_MSRV"
elif [[ "$REJECT_MODE" == drift && "${*: -1}" == */package-qualification/attempt.*/Cargo.lock ]]; then
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
    if [[ "$REJECT_MODE" == selected-msrv ]]; then
        printf '%s\n' "$*" >> "$MSRV_EVENTS"
        if [[ "$*" == *' run '* ]]; then
            previous=""
            manifest=""
            for argument in "$@"; do
                if [[ "$previous" == --manifest-path ]]; then manifest="$argument"; break; fi
                previous="$argument"
            done
            [[ "$("$REAL_YQ" -p toml -o yaml -r '.package.rust-version' "$manifest")" == "$FIXTURE_MSRV" ]] || exit 97
        fi
        exit 0
    fi
    echo 'unexpected compilation after invalid admission' >&2
    exit 99
fi
if [[ "$REJECT_MODE" == duplicate && "$*" == *' metadata '* && "$*" == *'/package-qualification/attempt.'* ]]; then
    "$REAL_CARGO" "$@" | jq '.packages += [.packages[] | select(.name == "ic-memory")]'
else
    exec "$REAL_CARGO" "$@"
fi
STUB
cat > "$fixture/bin/rustup" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$REJECT_MODE" == selected-msrv ]]; then
    [[ $# == 4 && "$1" == run && "$3" == rustc && "$4" == --version ]] || exit 99
    printf '%s\n' "$2" >> "$MSRV_ADMISSION_EVENTS"
    printf 'rustc %s\n' "$2"
else
    exec "$REAL_RUSTUP" "$@"
fi
STUB
chmod +x "$fixture/bin/cargo" "$fixture/bin/yq" "$fixture/bin/rustup"
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
        duplicate) expected='archive graph must use exactly the extracted Persistence packages' ;;
    esac
    rg -q "$expected" "$fixture/$mode.log" || { cat "$fixture/$mode.log" >&2; exit 1; }
    echo "Rejected $mode before compilation"
done
# Alter only the fixture's catalog observation. Real archive extraction and
# graph admission run; compiler checks/build/run effects are substitutes.
export FIXTURE_MSRV=1.90.0 MSRV_EVENTS="$fixture/msrv-events" MSRV_ADMISSION_EVENTS="$fixture/msrv-admission"
REJECT_MODE=selected-msrv PATH="$fixture/bin:$PATH" bash scripts/ci/package-workspace.sh "$pin" --allow-dirty > "$fixture/msrv.log" 2>&1
printf '%s\n' "$pin" "$FIXTURE_MSRV" > "$fixture/expected-admission"
cmp "$fixture/expected-admission" "$MSRV_ADMISSION_EVENTS"
printf '+%s\n' "$pin" "$pin" "$pin" "$pin" "$FIXTURE_MSRV" "$pin" > "$fixture/expected-compilers"
cut -d ' ' -f 1 "$MSRV_EVENTS" > "$fixture/observed-compilers"
cmp "$fixture/expected-compilers" "$fixture/observed-compilers"
echo 'Archive fixture metadata and compiler dispatch follow the admitted catalog MSRV (substitutes only).'
shasum -a 256 -c "$PREPARED_ARCHIVES"
