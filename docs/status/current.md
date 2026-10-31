# Current handoff

This batch, prepared from released `c2ee327` (0.57.0), adopts Platform 0.11.1,
Wire 0.10.0, Metrics/Logging 0.21.0 and Tooling 0.72.0, extending the preceding
uncommitted Wire adoption. Both maintained catalogs/locks and the separately
prepared Build/Testkit CLI pins are aligned. No commit, own-version bump,
release, publication or push ran. The pending changelog remains 0.58.0 because
the stable adapters expose new Wire error identities and Metrics types change
Rust dependency identity.

The root lock removes older native Platform and Wire providers. Production has
one Platform runtime owner and one core Wire provider; the native meter and
Metrics metadata now share them. Wire endpoint/platform-types features remain
disabled. Stable-structures stays 0.7.2; PocketIC stays client/server 16.1.0.
Stored bytes, arithmetic, Logging semantics, bounds and custody are unchanged.
No reset or retained-data migration is needed. Shared Tooling and common catalogs
are unchanged; prior tool generations and attempts remain retained.

Evidence under `/tmp/ic-persistence-deps13/` includes full locked metadata for
both graphs, production graph/pins, strict affected library/test/helper and
independent-runner Clippy, Rust 1.91 library/test/helper checks, formatting and
local links. All 187 Memory, 12 Logging Stable, 29 Metrics Stable and 69 release
helper tests pass (eight explicit Memory ignores). The selected bootstrap
recovery function passes with the real 0.72 editor and substituted Cargo effects.
An initial offline server check refused a concurrently held tool-store lock;
its retry passes after the owner finished. Both prepared executables are admitted.

The three rebuilt original Wasm fixtures pass installed IO/frame refusals,
upgrade rollback, Storage reopen, manager faults, custody and Logging budgets
using Testkit 0.72. Source checks match after execution. Inputs, tools, binaries
and logs remain under `target/qualification/runtime/attempt.atoCde/`.
See [the installed record](../runtime-io-qualification.md#platform-011-and-tooling-072-adoption--2026-10-30).
Full release gates, optimizer variants and native macOS were not rerun.

Canonical owners: [ledger](../current-ledger.md),
[Logging](../../crates/ic-logging-stable/README.md#upgrade-disposition),
[host support](../host-support.md), [release contract](../releases.md),
[adoption boundaries](../shared-tooling-adoption.md) and [notes](../../CHANGELOG.md).
