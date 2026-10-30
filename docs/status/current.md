# Current handoff

This work record covers adoption prepared from 0.56.4 `08e815d`, preserving the
preceding cleanup edits and incoming root Wire 0.7.25 lock update. Git status and
history own live checkout/release state. Platform 0.9.1, Logging/Metrics 0.18.1
and reviewed Shared Tooling 0.27.11 are selected; Build/Testkit executable pins,
common tool catalogs and the qualified substrate/client/server identities stay
unchanged. Both maintained dependency caches were explicitly prepared; checks ran
offline. No commit, own-version bump, release, publication, push or deployed reset
ran. Pending notes are combined under 0.57.0 because Logging adoption is breaking.

The maintainer explicitly retired old Logging checkpoints. Current discriminator
3 carries the text ceiling; normal load/save/remove refuse old rows. Hosts own
[explicit retirement or fresh allocations](../../crates/ic-logging-stable/README.md#upgrade-disposition).
Saving uses borrowed checkpoints, with prior-row continuity and unchanged-write
checks intact. Metrics durable bytes remain unchanged. Fixture support uses
ordinary module discovery; source lint exceptions use scoped expectations.

Evidence is retained under `/tmp/ic-persistence-adopt7/`: focused library tests,
strict native/Wasm/host Clippy, Rust 1.91 Wasm checking, Logging doctests,
consumer release-adapter substitutes, graph/pin checks and snapshot source/mode
verification pass. Installed evidence has separate Platform and Logging phases,
including text eviction/refusal, forged usage, rollback and reopen. See the
[source-bound installed record](../runtime-io-qualification.md#platform-09-and-logging-018-adoption--2026-10-30).
Formatting, package-support inventories and documentation links pass. Full release
gates, optimizer variants and macOS were not rerun. No speedup is claimed.

The follow-on patch update is recorded under `/tmp/ic-persistence-adopt8/`.
The incoming root lock already selected these patch identities and was preserved;
minimum requirements and the independent Platform lock now agree. All 41
Metrics/Logging adapter tests and a linked Logging Wasm build pass.
Production storage-only and native PocketIC graphs omit the runtime guard; the
CDK fixture selects it. Published Platform/macros and Observability source trees
compare byte-identical to the previously qualified versions. Catalogs/lockfiles,
formatting and documentation checks pass. Earlier installed evidence keeps its
original package identities; installed runtime and full release suites were not
rerun for this feature-guard/publication-only patch update.

Canonical owners: [ledger](../current-ledger.md), [host support](../host-support.md),
[release contract](../releases.md), [adoption boundaries](../shared-tooling-adoption.md)
and [release notes](../../CHANGELOG.md). Implementation covers
[#141](https://github.com/dragginzgame/ic-persistence/issues/141); remote closure and
release delivery were not performed.
