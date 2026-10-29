# Current handoff — Logging Stable qualification

This isolated contribution starts from released Persistence 0.54.0 source
`9ca2e8e39f8ab5d1b8501e4d843b25c614719a13`. The maintainer selected a separate
adapter PR, [#138](https://github.com/dragginzgame/ic-persistence/pull/138).
The normal checkout's unrelated scaffold/tooling edits remain untouched.
Workspace versions remain 0.54.0; compatible candidate notes are 0.54.1.
Logging Stable remains unpublished. Version projection includes four members;
publication and archive receipts retain the original three-package roster.

## Checkpoint boundary

`ic-logging-stable` checkpoints complete canonical Logging state through one
host-opened governed Storage map. It owns bounded V1 bytes, corruption admission
and checkpoint continuity. Logging owns restoration, append, retention and paging.
The [package guide](../../crates/ic-logging-stable/README.md) defines byte layout,
bounds and host integration. Hosts own allocation declarations, backing custody,
epochs, clocks, authorization, lifecycle and checkpoint/retention cadence.

Loads distinguish absence from valid empty/exhausted histories. Empty checkpoints
retain time and sequence watermarks. Save rejects regressed counters, changed
limits/identity, rewritten issued rows and resurrection of pruned rows. Identical
checkpoints skip stable writes. Explicit removal checks the old identity/limits;
reset/generation authority stays with the host. Diagnostic history never
authorizes retries or proves paid effects.

The adapter admits at most 1,024 configured entries, 4,096 message bytes and 128
bytes per label, with a 65,536-byte complete record ceiling. Maximum settings need
not fit simultaneously. Decoding bounds bytes/counts before constructing rows
and calls canonical Logging restoration. Malformed or unknown state never falls
back to reset/initialization.

## Published dependency evidence

The root lock selects registry Logging 0.17.2, published 2026-10-29, checksum
`8ead6c0fa5b61da08c5238e61ad57e9acf7c42a2af35c41657be5e60ea93e88f`.
No path patch, synthetic checksum or candidate API substitutes for it. Both locks
retain every previous package identity; additions supply the adapter/probe and
its shared native Wire protocol. Existing Memory, Storage and Metrics Stable
runtime APIs/formats, Shared Tooling snapshot and stable-structures 0.7.2 are
unchanged. Library Platform remains 0.7.4; the native runner remains 0.7.3.

Strict Clippy passes for the adapter, Wasm probe, release helper and native host.
Rust 1.91 passes seven adapter tests, a complete reopen doctest, a raw-backing
compile-fail proof and Wasm probe compilation. Native host tests pass (four;
two pre-existing installed tests remain ignored). Production graph/pin checks,
66 release-helper regressions, release adapter substitutes and real offline
interrupted-state bootstrap pass. The final helper batch reuses 65 passing tests
and reruns the corrected expected-surface fixture; original failure logs remain.
Bootstrap checks cover all four members and retain input checksums without
changing tracked inputs. Logs use `/tmp/ic-logging-stable-registry-*.log`,
`/tmp/ic-logging-stable-roster-*.log`, `/tmp/ic-logging-stable-host-*.log` and
`/tmp/ic-logging-stable-release-adapters-3.log`.

## Installed Logging evidence

`make test-runtime-logging` passed on Linux using pinned Rust 1.99.1, original
Wasm, Testkit CLI 0.69.9 and the prepared PocketIC 16.1.0 client/server pair.
The fixture uses host-owned governed Memory/Storage and method-bound native Wire
with admission before copying request bytes. It verifies query checkpoint
non-persistence, trapped update and post-upgrade rollback, clean upgrade reopen,
timestamp refusal and sequence/time watermarks after pruning every retained row.

Evidence is `target/qualification/runtime/attempt.2c23BD/`, with input hashes,
original Wasm and runtime/server logs. Wasm SHA-256:
`bfda93e071ad82c18042089eba665e5e82beb76b06ac04ae00135b020ba68c9c`.
The command log is `/tmp/ic-logging-stable-installed.log`. The full runtime runner
now includes Logging for original/O3/Os/Oz variants; only the focused original
Logging probe ran locally. This is no IC cost measurement or Canic/Jobs acceptance.

The earlier `/tmp/ic-logging-stable-candidate` fixture and logs retain their
candidate-source identity. They are superseded for registry admission, not
rewritten as published-dependency evidence. Preserve unrelated recovery records
and evidence in the normal checkout. Full validation, native macOS, the optimized
runtime matrix, merging and release/publication remain separate work.
