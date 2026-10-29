# Current handoff — installed Logging qualification

This contribution incorporates released Persistence 0.54.1 source
`447a9bb272ffa4551fe6c6ad7f7d1211121f99f0`, which integrated the Logging Stable
adapter and four-member release roster. Workspace versions remain 0.54.1; the
new compatible notes are 0.54.2. Logging Stable remains unpublished. The existing
three packages retain archive/publication authority. No new release is selected.

[PR #138](https://github.com/dragginzgame/ic-persistence/pull/138) now adds installed
Logging qualification, its bounded fixture protocol and release-helper regression
coverage. The isolated checkout is `/tmp/ic-logging-stable-20261029`; the normal
Persistence checkout is not edited by this contribution. The incoming 0.54.1
handoff is retained at `/tmp/ic-logging-stable-main-0541-handoff.md`, including
its earlier `/tmp/ic-persistence-logging-integration.k23sYP/` evidence references.
Preserve existing journals, receipts and recovery archives.

## Retained package boundary

The [Logging Stable guide](../../crates/ic-logging-stable/README.md) owns bounded
V1 load/save/remove checkpoints over host-opened governed Storage maps. Published
Logging 0.17.2 owns canonical restoration, append, retention and paging. Hosts own
allocation, identity, clocks, authorization, lifecycle and checkpoint cadence.
Empty/exhausted histories retain watermarks. Malformed or regressing state never
falls back to reset; identical checkpoints skip writes. No production adapter,
Memory, Storage or Metrics Stable runtime source/API/format changes in this batch.

The root graph retains main's Platform 0.7.6 and native host tooling 0.69.12
selections; Build/Testkit CLI selections remain 0.69.11. The independent runner
retains Platform 0.7.3, native Testkit 0.69.9 and PocketIC client 16.1.0. Every
existing package identity in both main-branch locks is preserved. Additions
supply the fixture's CDK and shared native Wire protocol. Shared Tooling remains
reviewed 0.27.6 and the stable-structures substrate remains 0.7.2.

## Focused qualification

On the reconciled 0.54.1 graph, strict native adapter, Wasm probe and release-helper
Clippy pass. Rust 1.91 passes all seven adapter tests, both doctests and Wasm
probe compilation. The two affected release-helper tests pass: projection of all
four versions with only three receipt archives, and inherited manifest identity
admission including the unpublished member. The independent native host source
and lock are unchanged from its four passing tests/two existing ignored installed
tests and strict Clippy proof earlier in this contribution.

The reconciled graph also passes pins, production/dependency boundaries, release
adapter substitutes and real offline bootstrap for both interrupted metadata
states. Formatting, selected ShellCheck, documentation links, licenses and diff
checks pass. The unchanged shared snapshot retains its verified 123-file identity.

Current logs use `/tmp/ic-logging-stable-merged-*.log`. The earlier 66-test helper
batch on the 0.54.0 graph remains separately identified in
`/tmp/ic-logging-stable-roster-*.log`; it is not relabelled as a full 0.54.1 run.

## Installed original-Wasm proof

`make test-runtime-logging` passes on Linux with Rust 1.99.1, original Wasm,
Testkit CLI 0.69.11 and the prepared PocketIC 16.1.0 server. Evidence remains at
`target/qualification/runtime/attempt.pUTvdW/`, including input hashes, Wasm and
runtime/server logs. Wasm SHA-256:
`109daef205312af64ae8e656bf24d300fc85e072bf738c513788740709c4ec0a`.
The command log is `/tmp/ic-logging-stable-merged-installed.log`.

The fixture checks query checkpoint non-persistence, trapped update and
post-upgrade rollback, successful upgrade reopen, timestamp refusal, sequence/time
watermarks after pruning every row and Wire method identity refusal. Hosts own
the governed runtime, map and lifecycle; request admission precedes byte copying.
The full runtime runner now includes Logging in original/O3/Os/Oz variants;
only the focused original Logging probe ran locally. Native macOS, the optimized
matrix, Canic/Jobs acceptance and cost measurements remain separate work.

Earlier evidence at `target/qualification/runtime/attempt.2c23BD/` retains its
0.54.0/CLI 0.69.9 identity. `/tmp/ic-logging-stable-candidate` and its logs retain
their pre-publication source-candidate identity. None replaces the current graph's
proof. Review/CI, merge and eventual adapter publication remain with their owner.
