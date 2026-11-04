#!/usr/bin/env bash
# Shared companions: scripts/ci/fixture-lifecycle.sh scripts/ci/prepare-script-fixture.sh scripts/ci/run-validation-targets.sh scripts/ci/validation-evidence.pl scripts/ci/fixtures/validation-evidence-caller.rs rust-toolchain.toml
set -euo pipefail
selection=all
if [[ $# != 0 ]]; then
    [[ $# == 2 && $1 == --case && $2 == utf8 ]] || { echo 'usage: test-validation-evidence.sh [--case utf8]' >&2; exit 2; }
    selection=utf8
fi
root="$(cd -P "${BASH_SOURCE[0]%/*}/../.." && pwd -P)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
mkdir -p "$root/target"
fixture="$(fixture_create "$root/target/validation-evidence.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Validation evidence fixture retained"' EXIT
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset VALIDATION_INPUT_ROOT VALIDATION_INPUT_BASELINE VALIDATION_RUNNER_SNAPSHOT_PATH GITHUB_STEP_SUMMARY
source_root="$fixture/source with spaces"
[[ $selection != utf8 ]] || source_root+=$'\316\273'
mkdir -p "$source_root/ci"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$source_root" \
    scripts/ci/run-validation-targets.sh scripts/ci/validation-evidence.pl
cd "$source_root"
helper="$PWD/scripts/ci/validation-evidence.pl"
git init -q
git config user.name Fixture
git config user.email fixture@example.invalid
git config core.hooksPath /dev/null
git config commit.gpgsign false
git config tag.gpgsign false
cat > Makefile <<'MAKE'
.PHONY: lint client runtime
lint:
	@echo lint >> trace
client:
	@echo client >> trace
	@test ! -f fail-client
runtime:
	@echo runtime >> trace
	@test ! -f fail-runtime
MAKE
printf 'fixture source\n' > source.txt
jq -cnS '{schema:1,groups:["client","runtime"],stages:["lint","client","runtime"]}' > ci/validation.json
jq -cnS '["runtime","client"]' > ci/groups.json
jq -cnS '{host:"fixture-native",configuration:{features:["default"]}}' > context.json
git add Makefile source.txt ci context.json
git commit -qm baseline
git tag -a client-v1.0.0 -m baseline
baseline="$(git rev-parse HEAD)"
printf 'new source\n' >> source.txt
git add source.txt
git commit -qm changed
export RELEASE_SOURCE RELEASE_GROUP=client
RELEASE_SOURCE="$(git rev-parse HEAD)"
refuse() {
    if "$@" > "$fixture/refused.stdout" 2> "$fixture/refused.stderr"; then
        echo 'unexpected validation evidence admission' >&2; exit 1
    fi
    [[ ! -s "$fixture/refused.stdout" ]]
}
if [[ $selection == all ]]; then
    perl "$helper" baseline "$PWD" "$RELEASE_SOURCE" refs/tags/client-v1.0.0 > baseline.json
    jq -e --arg base "$baseline" '.available and .baseline == $base and .changes == ["source.txt"]' baseline.json >/dev/null
    perl "$helper" baseline "$PWD" "$RELEASE_SOURCE" refs/tags/missing > absent.json
    jq -e '.available == false' absent.json >/dev/null
    git tag lightweight
    perl "$helper" baseline "$PWD" "$RELEASE_SOURCE" refs/tags/lightweight > lightweight.json
    jq -e '.available == false' lightweight.json >/dev/null
    perl "$helper" baseline "$PWD" "$baseline" refs/tags/client-v1.0.0 > equal.json
    jq -e '.available and .changes == []' equal.json >/dev/null
    git tag -a future -m future
    perl "$helper" baseline "$PWD" "$baseline" refs/tags/future > future.json
    jq -e '.available == false and .reason == "baseline is not an ancestor"' future.json >/dev/null
    perl "$helper" roster "$PWD" ci/validation.json ci/groups.json > roster.txt
    [[ "$(cat roster.txt)" == $'lint\nclient\nruntime' ]]
    jq -cnS '["client"]' > ci/wrong-groups.json
    refuse perl "$helper" roster "$PWD" ci/validation.json ci/wrong-groups.json
    jq -cnS '["client","client","runtime"]' > ci/wrong-groups.json
    refuse perl "$helper" roster "$PWD" ci/validation.json ci/wrong-groups.json
fi
make_plan() {
    jq -cnS --arg source "$RELEASE_SOURCE" --arg group "$RELEASE_GROUP" --arg scope "$1" \
        '{schema:1,source:$source,group:$group,scope:$scope,policy:"ci/validation.json",
          inputs:["Makefile","source.txt","context.json"],stages:[
          {target:"lint",required:true,reason:"common policy"},
          {target:"client",required:true,reason:"client contract"},
          {target:"runtime",required:($scope == "workspace"),reason:"runtime contract"}]}' > plan.json
}
run_plan() {
    export VALIDATION_REPOSITORY_ROOT="$PWD" VALIDATION_LOG_DIR="$PWD/logs/$1"
    export VALIDATION_FAILURE_LOG_DIR="$fixture/failures"
    bash scripts/ci/run-validation-targets.sh --plan plan.json > "$fixture/$1.log" 2>&1 || return "$?"
    local runs=("logs/$1"/validation.*)
    [[ ${#runs[@]} == 1 ]]
    run="${runs[0]}"
}
verify() { perl "$helper" verify "$PWD" plan.json "$RELEASE_SOURCE" "$RELEASE_GROUP" "$run" context.json; }
check() { perl "$helper" check "$PWD" plan.json "$RELEASE_SOURCE" "$RELEASE_GROUP" "$run" context.json proof.json; }
if [[ $selection == utf8 ]]; then
    make_plan group
    jq -cS '.stages[0].reason = "Latin-1 \u00e9" | .stages[1].reason = "Quote \u2019" |
        .stages[2].reason = "Excluded \u96ea"' plan.json > utf8-plan.json
    mv utf8-plan.json plan.json
    : > fail-runtime
    run_plan utf8
    verify > proof.json
    check > checked.json
    cmp proof.json checked.json
    jq -e '[.stages[].state] == ["PASS","PASS","not-applicable"]' proof.json >/dev/null
    cp "$run/plan-events.tsv" "$fixture/events.saved"
    printf '\377' >> "$run/plan-events.tsv"
    refuse verify
    cp "$fixture/events.saved" "$run/plan-events.tsv"
    cp "$run/0.log" "$fixture/log.saved"
    printf 'changed UTF-8 fixture log\n' >> "$run/0.log"
    refuse check
    cp "$fixture/log.saved" "$run/0.log"
    check > recovered.json
    cmp proof.json recovered.json
    fixture_complete=true
    printf 'UTF-8 reasons/log paths and corrupted evidence refusal passed\n'
    exit 0
fi
make_plan group
: > fail-runtime
run_plan narrow
verify > proof.json
check > checked.json
cmp proof.json checked.json
jq -e '.schema == 1 and .kind == "group-validation-evidence" and
    [.stages[].state] == ["PASS","PASS","not-applicable"]' proof.json >/dev/null
# Tooling-style pointer shell adapter: exactly the selected run, no directory glob.
jq -cnS --arg run "$run" --arg source "$RELEASE_SOURCE" --arg group "$RELEASE_GROUP" \
    '{schema:1,source:$source,group:$group,directory:$run}' > pointer.json
perl "$helper" check "$PWD" plan.json "$RELEASE_SOURCE" "$RELEASE_GROUP" \
    "$(jq -er .directory pointer.json)" context.json proof.json > pointer-proof.json
cmp proof.json pointer-proof.json
# Storage-style native caller: process status and canonical machine result cross
# the Rust boundary without implementing another event/timing protocol.
compiler="$(yq -r '.toolchain.channel' "$root/rust-toolchain.toml")"
RUSTUP_AUTO_INSTALL=0 rustc +"$compiler" --edition 2024 \
    "$root/scripts/ci/fixtures/validation-evidence-caller.rs" -o "$fixture/native-caller"
"$fixture/native-caller" "$helper" check "$PWD" plan.json "$RELEASE_SOURCE" "$RELEASE_GROUP" "$run" context.json proof.json > native-proof.json
cmp proof.json native-proof.json
# Preserve an embedded native-adapter envelope separately from Shared's proof.
# This shape follows Storage's GroupValidation boundary; product fields are not
# reinterpreted by Shared. Only the proof component replaces generic parsing.
jq -cnS --arg path plan.json --arg run "$run" --slurpfile plan plan.json \
    --slurpfile context context.json --slurpfile proof proof.json \
    '{schema:1,plan_path:$path,plan:$plan[0],context:$context[0],runner:$run,
      source_inputs:$proof[0].inputs,files:$proof[0].files,shared_evidence:$proof[0]}' > native-envelope.json
jq -cS '.shared_evidence' native-envelope.json > native-extracted.json
"$fixture/native-caller" "$helper" check "$PWD" \
    "$(jq -er .plan_path native-envelope.json)" "$RELEASE_SOURCE" "$RELEASE_GROUP" \
    "$(jq -er .runner native-envelope.json)" context.json native-extracted.json > native-recovered.json
cmp proof.json native-recovered.json
# Preserve the original proof when logs or context are changed.
cp "$run/0.log" "$fixture/log.saved"
printf 'tampered log\n' >> "$run/0.log"
refuse check
refuse "$fixture/native-caller" "$helper" check "$PWD" plan.json "$RELEASE_SOURCE" "$RELEASE_GROUP" "$run" context.json proof.json
cp "$fixture/log.saved" "$run/0.log"
cp context.json "$fixture/context.saved"
jq -cnS '{host:"other"}' > context.json
refuse verify
cp "$fixture/context.saved" context.json
refuse perl "$helper" verify "$PWD" plan.json "$RELEASE_SOURCE" runtime "$run" context.json
refuse perl "$helper" verify "$PWD" plan.json "$baseline" "$RELEASE_GROUP" "$run" context.json
# Every altered required artifact must refuse, without emitting partial JSON.
for name in plan-events.tsv timings.tsv plan-selection.tsv parsed-inputs.json.plan.json \
    parsed-inputs.json.policy.json plan-inputs.json parsed-inputs.json inputs.json \
    0-before-inputs.json 1-after-inputs.json; do
    cp "$run/$name" "$fixture/artifact.saved"
    printf 'corrupt\n' >> "$run/$name"
    refuse verify
    cp "$fixture/artifact.saved" "$run/$name"
    mv "$run/$name" "$run/$name.absent"
    refuse verify
    mv "$run/$name.absent" "$run/$name"
done
cp "$run/plan-events.tsv" "$fixture/events.saved"
for mutation in 's/\tPASS\t/\tFAIL\t/' 's/\tPASS\t/\tblocked\t/' 's/\tnot-applicable\t/\tPASS\t/'; do
    perl -pe "$mutation" "$fixture/events.saved" > "$run/plan-events.tsv"
    refuse verify
done
cat "$fixture/events.saved" "$fixture/events.saved" > "$run/plan-events.tsv"
refuse verify
cp "$fixture/events.saved" "$run/plan-events.tsv"
cp "$run/timings.tsv" "$fixture/timings.saved"
tail -n 1 "$fixture/timings.saved" >> "$run/timings.tsv"
refuse verify
cp "$fixture/timings.saved" "$run/timings.tsv"
# Valid rows in the wrong order cannot supply the original execution.
perl -e 'my @r=<>; print shift @r; print reverse @r' "$fixture/timings.saved" > "$run/timings.tsv"
refuse verify
cp "$fixture/timings.saved" "$run/timings.tsv"
perl -pe 's{/[0-9]+\.log}{/../outside.log}' "$fixture/timings.saved" > "$run/timings.tsv"
refuse verify
cp "$fixture/timings.saved" "$run/timings.tsv"
mv "$run/0.log" "$run/log.saved"
ln -s log.saved "$run/0.log"
refuse verify
rm "$run/0.log"
mkfifo "$run/0.log"
refuse verify
rm "$run/0.log"
perl -e 'open my $f, ">", $ARGV[0] or die; seek $f, 67108864, 0 or die; print {$f} "x"' "$run/0.log"
refuse verify
mv "$run/log.saved" "$run/0.log"
mv "$run" "$run.saved"
ln -s "${run##*/}.saved" "$run"
refuse verify
rm "$run"
mv "$run.saved" "$run"
check > checked.json
cmp proof.json checked.json
# Real required failure and interruption cannot be promoted to a group proof.
: > fail-client
if run_plan failed; then exit 1; fi
runs=(logs/failed/validation.*); run="${runs[0]}"
refuse verify
rm fail-client fail-runtime
make_plan workspace
run_plan full
verify > full-proof.json
jq -e 'all(.stages[]; .state == "PASS")' full-proof.json >/dev/null
cp "$run/plan-events.tsv" "$fixture/full-events.saved"
sed '$d' "$fixture/full-events.saved" > "$run/plan-events.tsv"
refuse verify
cp "$fixture/full-events.saved" "$run/plan-events.tsv"
verify > recovered.json
cmp full-proof.json recovered.json
# Keep the narrow regression in complete producer qualification as well.
bash "$root/scripts/ci/test-validation-evidence.sh" --case utf8
fixture_complete=true
printf 'Validation evidence, roster, baseline and shell/native caller fixtures passed\n'
