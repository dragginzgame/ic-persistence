# Current handoff

This batch, prepared from `b281a60`, adopts Platform 0.10.0 in both maintained
catalogs and Metrics/Logging 0.19.0 in the library. Preceding Wire 0.8 and Tooling
0.71 adoption remains. Registry review confirms Tooling libraries and the
prepared Build/Testkit CLI pins are already at the latest published 0.71.0.
Shared Tooling and common tool catalogs are unchanged. No commit, own-version
bump, release, publication or push ran.

Metrics now shares Wire 0.8 metadata, removing the old Wire 0.7.25 copy. The
production graph has one Platform 0.10 runtime owner. Tooling's native test meter
retains its required non-runtime Platform 0.9.1 provider outside production Wasm.
Stable-structures remains 0.7.2 and PocketIC remains client/server 16.1.0.
Public source APIs, stored bytes and allocation custody are unchanged.

Evidence under `/tmp/ic-persistence-adoption11/` includes 187 Memory tests
(eight explicit ignores), 12 Logging Stable tests, 29 Metrics Stable tests,
strict affected Clippy, Rust 1.91 library checks, graph/pins and formatting.
The three rebuilt original Wasm fixtures pass installed IO/frame refusals,
upgrade rollback, Storage reopen, manager faults, custody and Logging budgets.
Exact inputs, tools, binaries and logs remain under
`target/qualification/runtime/attempt.4dqXS3/`; source checks match after the run.
The first sandbox attempt could not bind loopback and is retained separately;
the permitted retry passed. See [the installed record](../runtime-io-qualification.md#platform-010-and-observability-019-adoption--2026-10-30).

Previous release-helper and prepared Tooling evidence is retained under
`/tmp/ic-persistence-tooling10/` and the earlier installed record. It is not
relabelled as new Platform evidence. Full release gates, optimizer variants and
native macOS were not rerun. The previous Logging hard-cut disposition remains.

Canonical owners: [ledger](../current-ledger.md),
[Logging](../../crates/ic-logging-stable/README.md#upgrade-disposition),
[host support](../host-support.md), [release contract](../releases.md),
[adoption boundaries](../shared-tooling-adoption.md) and [notes](../../CHANGELOG.md).
