#!/usr/bin/env bash
# Exercise actual workflow scripts with substituted read-only Git observations.
set -euo pipefail
root="$(pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-persistence-ci-selection.XXXXXX")"
echo "Retained CI selection fixture: $fixture"
yq -o=json '.' .github/workflows/ci.yml > "$fixture/workflow.json"
jq -r '.jobs["change-scope"].steps[] | select(.id == "classify") | .run' "$fixture/workflow.json" > "$fixture/classify.sh"
jq -e '
  (.jobs["change-scope"].steps[0].with["fetch-depth"] == 0) and
  (.jobs["change-scope"].steps[1].env.CHANGE_HEAD == "${{ github.sha }}") and
  (.jobs["change-scope"].steps[1].env.CHANGE_REF | contains("github.base_ref")) and
  (.jobs["change-scope"].steps[1].env.CHANGE_BASE | contains("github.event.pull_request.base.sha")) and
  (.jobs.docs.needs == "change-scope") and
  (.jobs.docs.steps[-1].run == "make shared-tooling-check check-doc-links") and
  (.jobs["tooling-lint"].needs == null) and
  (.jobs["tooling-lint"].steps | any(.run == "make lint-tooling")) and
  (.on.pull_request | has("paths") or has("paths-ignore") | not) and
  (.on.push | has("paths") or has("paths-ignore") | not) and
  (.jobs.test.steps | any(.id == "evidence" and (.run | contains("make validate-toolchain")))) and
  (.jobs.runtime.steps | any((.run // "") | contains("make test-runtime"))) and
  (.jobs.msrv.steps | any((.run // "") | contains("cargo check --workspace --locked --offline --all-targets"))) and
  all(.jobs | to_entries[] | select(.key == "test" or .key == "runtime" or .key == "msrv");
    .value.needs == ["change-scope","docs"] and
    .value.strategy.matrix.os == ["ubuntu-24.04","macos-15","macos-15-intel"] and
    .value["runs-on"] == "${{ matrix.os }}" and
    (.value.steps | length) > 3 and
    all(.value.steps[:-1][];
      .if | startswith("${{ needs.change-scope.outputs.scope != '\''docs'\''")) and
    .value.steps[-1].if == "${{ needs.change-scope.outputs.scope == '\''docs'\'' }}")
' "$fixture/workflow.json" > /dev/null
# Read-only observations replace Git effects; no fixture commits/tags are created.
mkdir "$fixture/bin"
cat > "$fixture/bin/git" <<'GIT'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  cat-file) [[ "$FIXTURE_MODE" != missing-history ]] ;;
  merge-base) [[ "$2" == --is-ancestor ]] || printf '%s\n' "$CHANGE_BASE" ;;
  ls-tree) printf '100644 blob %s\tci/docs-only-paths.txt\0' "$CHANGE_BASE" ;;
  show)
    cat "$POLICY_SOURCE"
    if [[ "$FIXTURE_MODE" == policy-change && "$2" == "$CHANGE_HEAD:"* ]]; then printf 'another.md\n'; fi
    ;;
  diff)
    if [[ "$2" == --check ]]; then [[ "$FIXTURE_MODE" != whitespace-failure ]]; exit; fi
    paths=(README.md)
    mode=100644
    case "$FIXTURE_MODE" in
      code) paths=(crates/ic-memory/src/lib.rs) ;;
      mixed) paths+=(crates/ic-memory/src/lib.rs) ;;
      policy-path) paths=(ci/docs-only-paths.txt) ;;
      workflow) paths=(.github/workflows/ci.yml) ;;
      packaged) paths=(crates/ic-memory/README.md) ;;
      unknown) paths=(docs/unreviewed.md) ;;
      executable) mode=100755 ;;
      symlink) mode=120000 ;;
    esac
    for path in "${paths[@]}"; do
      printf ':100644 %s %s %s M\0%s\0' "$mode" "$CHANGE_BASE" "$CHANGE_HEAD" "$path"
    done
    ;;
  *) exit 97 ;;
esac
GIT
cat > "$fixture/bin/perl" <<'PERL'
#!/usr/bin/env bash
set -euo pipefail
case "$FIXTURE_MODE" in
  classifier-failure) exit 23 ;;
  invalid-output) echo unknown; exit 0 ;;
esac
exec "$REAL_PERL" "$@"
PERL
chmod +x "$fixture/bin/git" "$fixture/bin/perl"
REAL_PERL="$(command -v perl)"
POLICY_SOURCE="$root/ci/docs-only-paths.txt"
export REAL_PERL POLICY_SOURCE
export CHANGE_BASE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa CHANGE_HEAD=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
export CHANGE_EVENT=push CHANGE_REF=refs/heads/main
for scenario in docs code mixed policy-change policy-path workflow packaged unknown missing-history classifier-failure invalid-output executable symlink tag pull-request; do
    expected=full
    export CHANGE_EVENT=push CHANGE_REF=refs/heads/main
    case "$scenario" in
      docs) expected=docs ;;
      pull-request) CHANGE_EVENT=pull_request; expected=docs ;;
      tag) CHANGE_REF=refs/tags/v1.0.0 ;;
    esac
    export FIXTURE_MODE="$scenario" GITHUB_OUTPUT="$fixture/$scenario.outputs"
    PATH="$fixture/bin:$PATH" bash -e -o pipefail "$fixture/classify.sh" > "$fixture/$scenario.log" 2>&1
    [[ "$(cat "$GITHUB_OUTPUT")" == "scope=$expected" ]] || exit 1
    printf '%s: %s\n' "$scenario" "$expected"
done
export FIXTURE_MODE=whitespace-failure GITHUB_OUTPUT="$fixture/whitespace.outputs"
if PATH="$fixture/bin:$PATH" bash -e -o pipefail "$fixture/classify.sh" > "$fixture/whitespace.log" 2>&1; then
    echo 'whitespace failure reported success' >&2; exit 1
fi
# Execute the real docs job command, not a simulated native pass.
jq -r '.jobs.docs.steps[-1].run' "$fixture/workflow.json" > "$fixture/docs.sh"
bash -e -o pipefail "$fixture/docs.sh" > "$fixture/docs.log" 2>&1
echo 'Actual workflow selection, unchanged full matrices/steps, fail-closed cases and documentation checks passed.'
