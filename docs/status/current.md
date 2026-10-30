# Current handoff

The latest release is 0.56.0. This checkout remains at `956b2b9` with uncommitted
work; the working tree includes merged [PR #140](https://github.com/dragginzgame/ic-persistence/pull/140)
and its published Logging 0.17.5 minimum. The combined pending draft is 0.56.1.
All incoming dependency-lock edits were preserved exactly. No local staging,
commit, version bump, release, publication or deployed data reset ran in this batch.

## Ten requested complexity reductions

Release preparation borrows one source projection and captures live rollback
bytes in one plan. Artifact paths come from one admitted target observation per
qualification phase; digest refusal ordering, archive/receipt shapes and
root-manifest-last publication remain unchanged. The release adapter separates
source admission, artifacts, committed-release checks and Wasm checks into private
modules, without adding alternate workflows or caches across input changes.

Doctor uses named readable/unreadable recovery observations and named report
fields. Runtime lifecycle has one private committed payload/accessor; public
errors and const accessors remain. Histogram continuity separates monotonic
bucket additions from intervals. Logging compares contiguous retained history,
retaining watermark/regression checks. The ledger writer/reader share a private
Wire row schema; borrowed sequence serialization adds no collected row buffer.
Current golden fixtures pin the same bytes.

Metrics and runtime tests are grouped by contract with existing fixtures shared
in their parent modules. All 29 Metrics and 19 moved runtime scenarios/assertions
were compared against the original functions after formatting and path adjustment.
Earlier qualification prose is preserved in
[the history guide](../history/ledger-qualification.md), with links updated; the
[current guide](../current-ledger-qualification.md) contains current Wire proof.
No custody, corruption, recovery or stable-memory limits were removed.

## Focused evidence

Logs and incoming/merged patch baselines are retained under
`/tmp/ic-persistence-ten.xUlwfE/`. All 68 existing release-helper tests pass;
the additional coordinated-archive test separately proves unchanged receipts
with one target observation for all four packages. Memory passes 187 library
tests (eight explicit ignores), Metrics 29 and Logging 10, including a new rolling
retention/overlap refusal test. Strict affected-library/test/release-helper Clippy,
Rust 1.91 Wasm library compilation, pin and production-graph checks pass.
Formatting, documentation links and diff checks cover the final work.

Linux installed original-Wasm qualification also passes through the admitted
Testkit CLI/PocketIC pair: invalid IO, successful/failed upgrades, Storage reopen,
manager poisoning/rollback, custody and Logging watermarks. Exact source/tool/
server/runner/Wasm hashes and all logs remain under
`target/qualification/runtime/cleanup-ten.qDQNqi/`; source hashes match before
and after. See [the installed record](../runtime-io-qualification.md#complexity-cleanup-installed-qualification--2026-10-30).
This qualifies recorded working-tree inputs, not the clean release's old lock.
Optimizer variants, macOS, full release gates and performance comparisons were
not rerun or inferred. Earlier logs/archives keep their own source identities.

## Retained-data boundary

The [ledger contract](../current-ledger.md) remains Wire-only with `ICML` framing
and `ICMWIRE1` protected generations. Earlier CBOR images require host-owned
fresh/reset installation or a separately reviewed transition. No legacy decoder,
fallback or automatic reset is included; never clear only governance while
leaving application collection bytes behind.
