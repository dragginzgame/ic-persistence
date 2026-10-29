# Current handoff — Logging Stable candidate

This isolated draft starts from released 0.54.0 source
`9ca2e8e39f8ab5d1b8501e4d843b25c614719a13`. The maintainer selected a separate
adapter PR awaiting Logging 0.17.2. The normal Persistence checkout's uncommitted
scaffold, release-roster and tooling batch is untouched and is not copied here.
Package versions remain 0.54.0; compatible candidate notes are 0.54.1. Publication
stays disabled for Logging Stable. Existing publication packages remain unchanged.

## Candidate contract

`ic-logging-stable` checkpoints complete canonical Logging state through one
host-opened governed Storage map. It owns bounded V1 bytes, corruption admission
and checkpoint continuity, with no second append/retention/paging implementation.
The [package guide](../../crates/ic-logging-stable/README.md) owns byte layout,
bounds and host integration. Hosts own allocation declarations, backing custody,
log identity, clocks, authorization, lifecycle and checkpoint/retention cadence.

Loads distinguish absent rows from valid empty or exhausted histories. Empty
checkpoints retain time and sequence watermarks. Save rejects regressed counters,
changed limits/identity, rewritten issued rows and resurrection of pruned rows.
Identical checkpoints validate and skip stable writes. The same key is replaced;
pruning does not accumulate archival rows or discard watermark metadata. Explicit
removal checks the old identity and configuration; reset/generation authority is
host-owned. Diagnostic history never authorizes retries or proves paid effects.

The adapter accepts at most 1,024 configured entries, 4,096 message bytes and 128
bytes per label, with a 65,536-byte complete record ceiling. These are separate
bounds; maximum settings need not fit simultaneously. Decoding bounds lengths and
counts before constructing rows and always calls the canonical Logging restore
boundary. Unknown formats and malformed state have no reset/initialization fallback.

## Focused candidate evidence

`/tmp/ic-logging-stable-candidate` is an isolated two-member qualification
workspace containing the actual candidate adapter and Observability source. Only
fixture manifests adapt the candidate dependency: Logging 0.17.1 metadata with the
new uncommitted restore API is a workspace member, never a production path patch.
The adapter runtime source files match this draft byte-for-byte. Published
Memory/Storage 0.54.0 runtime source trees match the Persistence base exactly.
Candidate lock/source inputs and logs remain in that fixture and
`/tmp/ic-logging-stable-candidate-*.log`.

Strict native Clippy passes. Rust 1.91 passes seven native governed store/codec
regressions, a complete host reopen doctest, a raw-backing compile-fail proof and
Wasm compilation. Tests cover empty/exhausted reopen, lease custody, malformed
lengths/flags/UTF-8, identity/configuration and byte refusals, zero writes for
identical/refused checkpoints, pruned-row resurrection refusal and storage reuse.
These are candidate-source/native proofs, not published dependency qualification,
installed update/query/trap/upgrade acceptance or IC performance evidence.

## Explicit draft prerequisites

The root catalog requires Logging 0.17.2; the released 0.17.1 API cannot build this
adapter. Normal locked checks require the upstream release and a fresh root lock
selection. No checksum, source identity or lock entry is invented for an unreleased
package. The existing scaffold/release-roster work must be integrated before a
coordinated release; this draft does not duplicate its metadata transaction work.
The adapter remains unpublished until owning qualification and release integration.

No existing Memory, Storage or Metrics Stable API/schema/runtime source changed.
The independent runtime-qualification workspace, both original locks, shared
snapshot and underlying stable-structures 0.7.2 selection remain unchanged.
Full validation, installed-runtime qualification, native macOS and release effects
are unselected. Preserve original evidence and unresolved recovery records in the
normal checkout; this draft does not replace or relabel them.
