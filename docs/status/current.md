# Current handoff

The checkout is based on the maintainer's scaffold/tooling commit
`bf38fe0`, following released 0.54.0 source
`9ca2e8e39f8ab5d1b8501e4d843b25c614719a13`. Pending notes remain 0.54.1;
package metadata remains 0.54.0. No staging, commit, push, merge or release effect
was performed during the integration below.

## Logging integration

[PR #138 — Prepare governed stable checkpoints for IC Logging](https://github.com/dragginzgame/ic-persistence/pull/138)
at `7a871520d1aec08ed6040d07a0b55e7d9bdfc65e` is combined locally with the
existing scaffold, release rosters and host tooling work. The working tree now
contains the adapter implementation, not just its original empty scaffold.

The root catalog and lock select published registry Logging 0.17.2, checksum
`8ead6c0fa5b61da08c5238e61ad57e9acf7c42a2af35c41657be5e60ea93e88f`.
No sibling source override, copied Logging engine or candidate checksum is used.
The incoming lock's host tooling 0.69.12 and Platform 0.7.6 selections are
preserved. The separate runtime-qualification lock is byte-identical to its input.
Build/Testkit CLI pins remain the separately prepared 0.69.11 selections.

Logging Stable owns bounded V1 checkpoint encoding and continuity checks over a
host-opened governed Storage map. Canonical Logging owns restore, append, paging
and retention. Empty/exhausted checkpoints retain both append watermarks;
regression, rewritten issued rows and resurrection of pruned rows refuse before
mutation. Identical saves perform no stable writes. Hosts own clocks, identity,
allocation, lifecycle, authorization and checkpoint cadence.

The logging package remains unpublished. All four members participate in version
projection and interrupted-release bootstrap. Only the original three crates
participate in archives, receipts and publication. Existing Memory/Storage/Metrics
formats, custody, Rust support, Shared 0.27.6 and PocketIC 16.1.0 are retained.

## Focused evidence and delivery boundary

Inputs and the preceding handoff are retained at
`/tmp/ic-persistence-logging-integration.k23sYP/`. Normal locked dependency preparation succeeds.
On the published graph, all seven Logging Stable tests and both doctests pass on
Rust 1.99.1 and actual Rust 1.91. Strict native all-target and Wasm library Clippy
pass after completing API error documentation. Formatting, pins, production graph,
licenses, documentation links and consumer release-adapter fixtures pass.
Adapter evidence is at `/tmp/ic-memory-release-adapters.lDwOWG/`.
Real offline bootstrap admits both interrupted metadata states with unchanged
fixture inputs at `/tmp/ic-persistence-bootstrap.wCaeVv/`.

The PR's original CI failure was dependency preparation before Logging 0.17.2
was available. Its remote head has not been updated with this local integration;
old failed CI is not new-source acceptance. The combined tree is ready for the
maintainer's contribution flow and fresh required CI. Full release, native macOS
and installed Logging update/query/trap/upgrade qualification remain unrun.
Publication of Logging Stable remains a separate explicit decision.

Preserve the earlier scaffold/bootstrap, native custody and adoption evidence
linked by the retained incoming handoff, along with all release journals,
receipts, failed attempts and archives. Runtime checkpoint/custody policy remains
as documented in the public Memory guide. No repository retirement retry is in scope.
