# Current handoff

This batch, prepared on the released `b20098d` source tree (0.58.0), adopts
published Platform 0.12.0 in both maintained catalogs and lockfiles, preserving
the preceding uncommitted Tooling 0.72.1 adoption and CLI pins. No Git or release
effects ran for this adoption. The pending changelog remains 0.58.1: the upstream
minor breaks only an optional gzip type identity that Persistence neither
selects nor exposes, so Persistence's public contract stays compatible.

Platform implementation, macro and runtime-guard source trees are identical
between upstream tags `v0.11.1` and `v0.12.0`. Its optional native gzip facade
updates Host Artifacts identity. Production Wasm has one Platform 0.12 runtime
owner; Tooling's native test meter retains a required non-runtime Platform 0.11.1
provider outside production. No unrelated dependency identity changed.
Stable-structures remains exactly 0.7.2 and PocketIC remains client/server 16.1.0.
Wire 0.10, Metrics/Logging 0.21, Shared Tooling and common catalogs are unchanged.
Stored bytes, bounds, arithmetic, logging semantics and allocation custody remain
unchanged; no reset, retained-data migration or source adaptation is required.

Evidence under `/tmp/ic-persistence-platform15/` includes registry/source review,
locked full metadata for both graphs, production graph/pins, strict native
Memory/Storage/Logging library/test Clippy, the Logging Wasm fixture Clippy, and
independent-runner Clippy. Twelve stable-cell, thirteen ledger-encoding and one
Storage allocation/reopen test pass (one explicit ledger-encoding ignore).
Rust 1.91 checks the Logging Wasm fixture and its Memory/Storage dependencies.
Formatting, documentation links and diff checks pass.

The preceding Tooling evidence stays under `/tmp/ic-persistence-tooling14/`:
prepared cached 0.72.1 Build/Testkit executables pass offline admission and the
selected real-editor bootstrap function passes with substituted Cargo effects.
Old tool generations and artifacts remain retained. Installed runtime and full
release suites were not repeated for identical upstream runtime/substrate source;
earlier installed records are historical and are not relabelled as Platform 0.12.

Canonical owners: [ledger](../current-ledger.md),
[Logging](../../crates/ic-logging-stable/README.md#upgrade-disposition),
[host support](../host-support.md), [release contract](../releases.md),
[adoption boundaries](../shared-tooling-adoption.md), [notes](../../CHANGELOG.md)
and [installed history](../runtime-io-qualification.md).
