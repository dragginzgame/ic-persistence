#!/usr/bin/env bash
# Exercise the real archive admission before any compilation, with bad observations.
set -euo pipefail
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-persistence-package-admission.XXXXXX")"
echo "Retained archive-admission fixture: $fixture"
mkdir -p "$fixture/bin"
REAL_CARGO="$(command -v cargo)"
REAL_YQ="$(command -v yq)"
export REAL_CARGO REAL_YQ
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
pin="$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)"
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
