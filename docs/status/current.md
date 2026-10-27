# Current handoff

The released baseline is 0.51.11 at `6aa75ac011f5131e19a03dfc704eb457e019b964`.
The pending compatible 0.51.12 batch implements
[#136 — Provide non-initializing StableCell loading for Blob singleton journals](https://github.com/dragginzgame/ic-persistence/issues/136)
and repairs macOS formatter admission for
[#135 — Adopt shared formatter orchestration without dropping package-consumer formatting](https://github.com/dragginzgame/ic-persistence/issues/135).
It adds an API and fixes contributor tooling without changing stored formats.
Package versions remain 0.51.11; all task changes are uncommitted. The incoming
staged `Cargo.lock` update to IC Build 0.69.1 remains byte-for-byte preserved,
including its staged state. See [pending notes](../changelog/0.51.md#05112---unreleased).

## Current changes

`StableCell::<Record, _>::load(runtime_memory)` takes a governed lease without a
caller default, sentinel record, `Default` or `Clone` bound. It reuses Memory's
existing envelope validator, now publicly exported as `stable_cell_payload_length`,
and the selected upstream Cell's canonical decoder. A private nonserializable
loading marker prevents initialization fallback before upstream can grow/write.
Successful cells cache the decoded value, retain the exclusive lease and support
existing `get`/`set` and reopen behavior. No duplicate Cell writer or substrate
fork is introduced.

Empty/foreign memory, unsupported headers and lengths beyond the allocation
return `StableCellPayloadError` without writes or growth. Codec/backing panics
retain upstream semantics; errors and ordinary unwinding release the consumed
lease. Installed Wasm traps follow host rollback semantics. Collection payloads
retain their existing size behavior, separately from the ledger's recovery
ceiling. Existing `new`, `init`, BoundedCell and V1 bytes are preserved.

The owning native adapter tests cover complete backing-byte preservation,
write/grow counters, malformed envelopes, decoder/backing panics, lease release,
single decoding without encoding, non-Default/non-Clone records, empty and large
payloads, canonical upstream interoperability, destructive creation and set/reopen.
The installed fixture now exercises cell loading and rejection alongside existing
collections. Its focused ignored test is documented in
[host support](../host-support.md); the normal installed runner also calls the
same qualification. No Blob application changes or resets are included.

The released formatter migration used a target-specific `override export`
assignment unsupported by macOS's GNU Make 3.81. Reclassifying caller compiler
values before target-specific export fixes parsing and preserves the selected
validation compiler. The consumer fixture checks every formatting pass with
conflicting environment and command-line compiler selections. Shared vendored
files remain unchanged.

## Focused qualification

Evidence is retained at `/tmp/ic-persistence-issue-136/`, with source/lock/Wasm
hashes in `inputs.sha256` and the exact installed artifact `storage-rust-1.91.wasm`.
All commands use the existing locked graphs and offline caches:

- Rust 1.99.1 Storage library adapter tests: 17 passed (`storage-native.log`).
- Rust 1.91.0 new cell adapter tests: 4 passed (`storage-msrv.log`).
- Memory's header-only length/ceiling rejection test passed (`memory-envelope.log`).
- Storage native/tests and actual Wasm example Clippy passed with warnings denied
  (`storage-clippy.log`, `storage-wasm-clippy.log`). The independent installed
  harness/test Clippy also passed (`runtime-clippy.log`).
- Rust 1.91.0 built the Storage qualification Wasm (`storage-wasm-msrv.log`).
  That exact artifact passed the focused installed adapter test against prepared
  PocketIC client/server 16.1.0 (`storage-installed.log`, `server.stdout`,
  `server.stderr`). It covers empty/foreign/unsupported/invalid envelopes,
  unchanged bytes/pages, lease release, decoder traps, loaded-cell set/reopen,
  update/upgrade rollback and successful upgrade restoration.
- Storage's declaration boundary, shared formatting, documentation links and diff
  checks pass. Both selected
  lockfiles remain unchanged from the incoming selections; the index is preserved.

The selected Testkit 0.69.0 release CLI initially lacked a local activation.
Explicit offline setup reused its existing admitted shared-store payload and
prepared the existing server selection; failed admission/setup logs and the
successful setup log remain retained. Compiler/tool/dependency selections were
not changed. This is Linux qualification; macOS and the full optimizer/release
matrix were not rerun for this focused fix.

Formatter portability evidence is retained at
`/tmp/ic-persistence-issue-135-make381/`. GNU Make 3.81 built from GNU's source
archive reproduces the released parse failure (`before.log`). The repaired source
passes snapshot admission and real formatting under that parser
(`native-make381.log`). Consumer release-adapter substitutes pass under GNU Make
3.81 and 4.3 (`adapters-make381.log`, `adapters-make43.log`), including compiler
selection, ordering, failure logs, jobserver handoff and substituted Git effects.
Bash syntax and ShellCheck pass. This Linux parser proof does not replace macOS
execution: the [released CI run](https://github.com/dragginzgame/ic-persistence/actions/runs/40538749483)
failed both macOS native/runtime jobs at Makefile parsing, so #135 remains open
pending delivery and hosted native acceptance. No rerun was dispatched.

## Delivery boundary

The supplier implementation is ready for review. Publication remains a separate
release step before Blob adopts the new API. Blob owns journal gateway, fence,
sync and callback qualification. No commit, tag, push, release or publication ran.
The public API and failure contract are documented in
[Storage's README](../../crates/ic-storage/README.md) and the
[ownership guide](../ownership.md).

The [released handoff](https://github.com/dragginzgame/ic-persistence/blob/6aa75ac011f5131e19a03dfc704eb457e019b964/docs/status/current.md)
retains prior formatter adoption and tooling evidence. Shared Tooling remains
0.26.26 at `5683060a5d8ce79af2fcfedb53b3eb24037fd064`; the reviewed
[adoption boundaries](../shared-tooling-adoption.md) remain active.

## Retained evidence and recovery

All original Memory/Storage issues, comments and PR histories remain preserved
through the [migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). Preserve the verified
backups at `/home/adam/projects/ic-persistence-retirement-2026-10-20/` and all
mirrors, exports and recovery plans. The original checkouts were retired with
maintainer authorization. Last recorded GitHub deletions were refused because
the CLI token lacked `delete_repo`; no deletion retry is part of this work.
Earlier package/runtime archives, receipts, failed attempts and logs retain their
original custody and source identities.
