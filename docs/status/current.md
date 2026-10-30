# Current handoff

The clean starting point was 0.56.3 `413cecc`. The six priority cleanup items
are implemented as uncommitted changes, with pending 0.56.4 notes. The four
optional audit items remain deferred. No package version, dependency selection,
commit, release, publication or push changed. Both catalogs and lockfiles are
unchanged.

## Completed cleanup

- Fixture diagnostics uses normal module discovery in its existing runtime
  namespace; its file moved byte-for-byte and test-support re-exports remain.
- ArtifactPaths owns validation receipts, package receipts and transformation
  inputs as well as archives/publication. Callers retain version admission and
  fresh target observations; receipt names and transformation custody are intact.
- Selected manager layout offsets, version and slot-domain bounds have one owner,
  with the same persisted values, read order and typed refusal precedence.
- Lease opening and Drop share bit-address calculation. Duplicate opens still
  refuse before metadata IO; marking follows successful admission and handle
  acquisition. Backing claim lifetime and poisoning are unchanged.
- Release fixtures share selected-candidate receipt reads. Corrupt bytes,
  retained evidence and mutation assertions remain explicit.
- The redundant Cargo-program check preceding is_cargo_call is removed; exact
  command assertions and effect guards remain.

## Focused evidence

Evidence and original source snapshots are retained under
`/tmp/ic-persistence-cleanup5/`. Strict Memory library/test/release-helper Clippy
passes. Memory passes 187 library tests (eight explicit ignores), including the
moved diagnostics, layout, corruption, growth and custody checks. All 69
release-helper tests pass, including path selection, receipt recovery and
retained-input admission. Formatting, selected documentation links and diff
checks pass.

A single Linux original-Wasm qualification passes through Testkit CLI 0.70.0
and PocketIC client/server 16.1.0. Refused frames, invalid IO, upgrade rollback,
Storage reopen, manager faults, custody and Logging watermarks pass. Exact
source/tool/binary hashes, commands and server/runtime logs remain under
`target/qualification/runtime/attempt.aCDH9i/`; source hashes match after the run.
See [the installed record](../runtime-io-qualification.md#runtime-bookkeeping-cleanup-qualification--2026-10-30).
No runtime source fix followed this run. Optimizer variants, macOS and full
release gates were not rerun; no speedup is inferred.

The [current ledger contract](../current-ledger.md) remains Wire-only. Earlier
CBOR images require host-owned fresh/reset installation or a separately reviewed
transition. No legacy decoder or automatic reset is introduced.
