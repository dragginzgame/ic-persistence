# Current handoff

The clean starting point was 0.56.2 `841d377`. All ten follow-on cleanup items
are implemented as uncommitted changes, with pending 0.56.3 notes. No package
version, dependency selection, commit, release, publication or push changed.
Both workspace catalogs and lockfiles are unchanged.

## Completed cleanup

Logging encodes borrowed rows without an intermediate row vector, using the
same checkpoint schema and bounded decoder. Release rollback and receipt schemas
have dedicated private owners, while named source/lock states make admission
choices explicit. Release fixtures share coherent source updates and recorded
command observations; intentional mismatches and exact-dispatch assertions stay
explicit.

Metric state/restoration is separated from continuity arithmetic. Governed
Storage implementations are grouped by collection, retaining the same public
re-exports. Logging tests are grouped into encoding, continuity and storage;
Memory fixture tests into setup, bounded loading and custody. No public API,
stored format, arithmetic check, lease rule or recovery boundary changed.

## Focused evidence

Logs, original source snapshots and preservation comparisons are retained under
`/tmp/ic-persistence-cleanup4/`. Strict Clippy for the affected libraries/tests and
release helper passes. Logging passes 10 library tests, Metrics 29, Storage 17,
Memory's selected fixture-cell group 11, and the release helper all 69. Affected
package doctests, including the governed-handle compile-fail examples, pass.
Formatting, selected documentation links and diff checks pass.

The comparison preserves all 21 moved test bodies, all moved Storage method
bodies and all four metric continuity bodies plus their three arithmetic helpers.
Receipt schemas/archive mapping, metadata rollback and the original error/source
projection also compare unchanged after visibility/formatting adjustments.
Frozen Logging bytes and maximum-row admission remain covered by existing tests.

Focused Linux original-Wasm Logging qualification passes with Testkit CLI 0.70.0
and PocketIC 16.1.0. It checks query non-persistence, update/upgrade rollback,
reopen and empty-history watermarks. Exact inputs, binaries and server/runtime
logs are retained in `target/qualification/runtime/attempt.5IgDJv/`; source hashes
match after the run. See [the installed record](../runtime-io-qualification.md#borrowed-logging-encoder-qualification--2026-10-30).
The first scratch launcher had a wrong pin-file path and stopped before server
startup; its failure evidence is preserved. No source fix followed the successful
installed run. Full runtime/release suites, optimizer variants and macOS were not
rerun. No speedup or new valid-IO measurement baseline is inferred.

## Retained-data boundary

The [ledger contract](../current-ledger.md) remains Wire-only with `ICML` framing
and `ICMWIRE1` protected generations. Earlier CBOR images require host-owned
fresh/reset installation or a separately reviewed transition. No legacy decoder,
fallback or automatic reset is included; never clear only governance while
leaving application collection bytes behind.
