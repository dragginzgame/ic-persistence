#!/usr/bin/env bash
# Exercise actual admission and archive entrypoint with compiler/Cargo substitutes.
set -euo pipefail
root="$(pwd -P)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/persistence-compilers.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Compiler admission fixture retained"' EXIT
mkdir -p "$fixture/bin" "$fixture/scripts/ci"
cp scripts/ci/check-validation-compilers.sh "$fixture/scripts/ci/"
printf '[workspace.package]\nrust-version = "1.91.0"\n' > "$fixture/Cargo.toml"
cat > "$fixture/bin/rustup" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 4 && "$1" == run && "$3" == rustc && "$4" == --version ]] || exit 99
printf '%s\n' "$2" >> compiler-events
[[ "$2" != "$MISSING_COMPILER" ]] || exit 31
printf 'rustc %s\n' "$2"
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >> cargo-events
exit 99
STUB
chmod +x "$fixture/bin/rustup" "$fixture/bin/cargo"
export PATH="$fixture/bin:$PATH"
cd "$fixture"
for missing in 1.99.1 1.91.0; do
    rm -f compiler-events
    if MISSING_COMPILER="$missing" bash "$root/scripts/ci/package-workspace.sh" 1.99.1 --allow-dirty > "$missing.log" 2>&1; then
        echo 'Archive packaging admitted a missing compiler' >&2; exit 1
    fi
    [[ ! -e cargo-events && ! -e target ]] || exit 1
    grep -F "Prepare required compiler first: rustup toolchain install --profile minimal $missing" "$missing.log"
    printf '1.99.1\n' > expected
    if [[ "$missing" == 1.91.0 ]]; then printf '1.91.0\n' >> expected; fi
    cmp expected compiler-events
done
rm compiler-events
MISSING_COMPILER=none bash scripts/ci/check-validation-compilers.sh 1.99.1 > success.log
printf '1.99.1\n1.91.0\n' > expected
cmp expected compiler-events
rm compiler-events
printf '[workspace.package]\nrust-version = "invalid"\n' > Cargo.toml
if MISSING_COMPILER=none bash scripts/ci/check-validation-compilers.sh 1.99.1 > invalid.log 2>&1; then exit 1; fi
[[ ! -e compiler-events && ! -e cargo-events ]] || exit 1
fixture_complete=true
echo 'Both compiler selections and early archive refusal passed without installs or builds.'
