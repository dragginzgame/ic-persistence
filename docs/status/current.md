# Current handoff

The release ledger's latest completed entry is 0.55.3. Package versions remain
0.55.3; the combined pending draft is now [0.56.0](../changelog/0.56.md) because
Memory's persisted ledger and error contracts change. No repository staging,
commit, release, registry upload, deployment or data reset ran here.

## Current Wire cut

The maintainer explicitly requested IC Wire and a hard cut. Logical ownership
records and the enclosing Cell value now use bounded Wire native encoding.
`ICML` identifies the Cell value; `ICMWIRE1` identifies the protected generation.
Wire owns byte/allocation/node/depth admission and exact consumption. Memory
retains numeric ceilings, checksum, generation, key/slot integrity, allocation
custody and same-format recovery. Logical key text borrows input until validation.

The main [catalog](../../Cargo.toml)/[lock](../../Cargo.lock) no longer declare or
resolve `serde_cbor` or `half`; the local CBOR walker and bounded writer are gone.
Platform 0.8.2 was already selected in the incoming dirty lockfile and is preserved.
The independent qualification [catalog](../../testing/runtime-qualification/Cargo.toml)
and [lock](../../testing/runtime-qualification/Cargo.lock) are unchanged. Its
external PocketIC client still depends on CBOR; no Memory package belongs to that
separate host graph.

The [current ledger contract](../current-ledger.md) owns adoption requirements.
Earlier CBOR images require fresh/reset installation or a separately reviewed
host transition before adoption. Never clear only governance while leaving
application collection bytes behind. No legacy reader, fallback or automatic
reset is provided. Raw-record errors expose `ic_wire::Error`; runtime encoding
refusals use `StableCellLedgerEncode`. Manager/Cell headers and application
collection formats are unchanged. Current fixtures were regenerated through the
production writers and re-encoding tests pin their bytes.

## Preserved cleanup and upstream prerequisite

Earlier local work reuses substrate Cell bounds, derives bootstrap manifests from
the package roster, shares preparation/final packaging commands, organizes process
execution and test contracts, caches roster parsing and uses slice reversal.
Host/adoption guidance identifies all four packages and authoritative selections.
Those changes remain part of the pending batch; prior evidence stays under
`/tmp/ic-persistence-cleanup.Vgl85C/` and `/tmp/ic-persistence-cleanup-five.R61Zf4/`.
The earlier 68 release-helper tests passed before this format cut; they are not a
live release qualification for the new codec.

Observability [PR #100](https://github.com/dragginzgame/ic-observability/pull/100)
provides `Log::into_snapshot` for the separate Logging round-trip cleanup. Its
isolated consumer proof and prepared patch remain in the earlier evidence folder.
Production Logging adoption still awaits a published release; this checkout adds
no path override or replacement implementation.

## Focused Wire evidence

Logs and selected source hashes are retained under `/tmp/ic-memory-wire-cut.9sHjJ0/`.
Memory's 187 unit tests and five diagnostic integration tests pass with
`test-support`; eight tests remain explicitly ignored, including measurements and
fixture regeneration. Explicit fixture regeneration passed separately. Storage's
17 library tests cover governed collection reopen and corruption/refusal paths.
Strict Memory library/test Clippy and selected Wasm diagnostic/IO-example Clippy
pass. Pin and production-graph checks admit the selected normal Wasm graph.

The published API doctests, actual Rust 1.91 Wasm compilation, final formatting,
local documentation links and diff checks are recorded with their final commands.
Earlier failed development checks remain retained, rather than relabelled as
passes. No full release suite, installed PocketIC IO/upgrade/rollback, macOS,
Wasm size or cost measurement is claimed. Installed qualification remains a
release prerequisite; earlier CBOR installation evidence does not qualify Wire.
