# Current handoff

Delivery selected: Persistence 0.62.0 (Memory/Storage), Metrics Stable 0.63.0 and
Logging Stable 0.62.0 to crates.io and GitHub `dragginzgame/ic-persistence`, `main`.
Standard full gates, final archive receipts and registry/tag readback own delivery
evidence under `target/delivery/wire-0.14-2026-11-01/`. Preparation descriptions
below predate this delivery selection; no old attempt or receipt is discarded.

Published on crates.io and GitHub: Persistence 0.61.1 (Memory/Storage), Metrics
Stable 0.62.1 and Logging Stable 0.61.1. Normal main is at `bef1cae`; the three
standard full release gates, final archive qualification, registry checksums,
published supplier requirements and annotated remote tags passed. Evidence:
`target/delivery/platform-wire-2026-11-01/verification.json` and retained gate logs.

Current local adoption selects Testkit Native 0.77.0 with its existing
`stable-memory` feature. Its Platform 0.17.0 selection removes the dev-only
Platform 0.16 copy; root production and native measurement callers now share the
same facade and stable-structures 0.7.2 trait identity. The catalog remains a
compatible range. Testkit CLI 0.76.0, server 16.1.0, the independent qualification
workspace, public persistence APIs and stored formats are unchanged.

The incoming root lock already selected Logging 0.23.7 and Metrics 0.24.6; those
edits are preserved outside this Native-only update. The original lock and patch
are retained under `target/testkit-native-0.77-adoption/`. No broad update or
executable-tool installation is selected. The pending Persistence notes include
this development dependency adoption; package versions, commits, release
execution and publication remain unchanged by this local work.

Focused Testkit Native qualification evidence is retained under
`target/testkit-native-0.77-adoption/`. Locked metadata confirms one Platform
0.17 provider. Affected Clippy, nine memory-accounting tests, one allocation
fixture test, Rust 1.91 native checks, production graph/pins and formatting/docs
checks passed. Full release and installed-runtime matrices were not replayed.
An initial Clippy invocation selected a
hyphenated example name; Cargo refused before compilation. Its log is retained,
and the actual target is `allocation_measurements`.

Wire/Derive 0.14.0 is now selected directly in both maintained catalogs/locks.
Public ledger and stable-adapter codec errors change Rust crate identity, so
pending notes select Persistence 0.62.0, Metrics Stable 0.63.0 and Logging Stable
0.62.0. The pending Testkit batch is carried forward in the core minor notes;
published history and current package versions remain unchanged. The core release
metadata now points to `releases/persistence/0.62.md`.

Encoded bytes, budgets, malformed-input behavior and retained-data contracts are
unchanged; no reset is required. Wire's optional platform-types/Endpoint features
remain unselected. Published Metrics 0.24.6 still requires Wire 0.13 for default
native schema support. Preserve that lane and feature behavior; no forced override
or schema-owner unification is selected. Production Wasm and the independent
native qualifier each admit one Wire 0.14 provider, while full native library
metadata contains the separate Metrics Wire 0.13 dependency. Both graph/Clippy,
production singleton and pin checks passed. Frozen/corrupt codec fixtures passed
33 tests and the qualifier's frame/argument fixtures passed three; existing
measurement/installed probes remain ignored. Rust 1.91 Wasm libraries and native
qualifier checks, formatting/docs, group planning and diff checks passed. The
inverse native graph confirms Metrics is the only Wire 0.13 dependency path.
Evidence remains under `target/wire-0.14-adoption/`; no release/publication or
installed-runtime matrix is selected by this adoption.

The published Platform 0.17/Wire 0.13.5 adoption retains stable-structures 0.7.2.
The native qualifier uses owned `ic-platform-pocket-ic` 0.17.0 with server 16.1.0;
Logging's qualification fixture uses Runtime/macros/guard 0.17.1 and Types 0.16.0.
Combined graph/pin/Clippy checks, Rust 1.91 builds, 72 Memory runtime tests and 20
Storage tests passed. Installed Linux IO, Storage, custody, Logging retention and
update/upgrade rollback passed with unchanged post-run inputs. Evidence remains
under `target/platform-0.17-adoption/`. Wire codec fixtures and earlier evidence
remain under `target/wire-0.13.5-adoption/`; native macOS and installed optimizer
variants remain separate acceptance.

The earlier API hard cut uses explicit Storage `create`/`load`/`reset`, one
bound-enforcing `StableCell`, parsed `StableKey` runtime lookup/open and nonzero
bucket configuration. Ledger errors retain typed decode sources. See
[the migration](../migration/0.61-public-api.md). Earlier focused and installed
proof remains under `target/hard-cut-api-2026-11-01/` and
`target/complexity-followup-2026-11-01/`.

The preceding hard-cut releases were delivered from the retained isolated
`target/delivery/hard-cut-observability-2026-11-01/checkout/` to preserve incoming
lock work. Their receipts/readback own completion; the original failed Persistence
0.61.0 plan is superseded failure evidence and must not be resumed. Autostash
backups and earlier release receipts/archives remain retained.

Build owns extracted archive graph admission and input custody; the consumer owns
its library/native/MSRV/Wasm roster. Shared's grouped bootstrap observer is adopted
from [PR #336](https://github.com/dragginzgame/shared-tooling/pull/336).
The merged destination-adoption check covers both independent graphs for
[Shared #335](https://github.com/dragginzgame/shared-tooling/issues/335).
Opt-in ShellCheck proof reuse retains complete gates and fresh group admissions.
PR #150 admits complete native macOS tool bytes within 64 MiB and preserves failed
logs/status. Native CI remains separate acceptance for
[Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334); retain the
earlier failed run 41314835837 and existing local proof. See
[adoption](../shared-tooling-adoption.md#exact-input-shellcheck-reuse).

The reviewed Shared snapshot remains
`9275bf0de39ed941d81c47bab97eef7f6fc38df2` (VERSION 0.27.24), with 81 roots and
136 payloads. Never patch vendored files or adopt dirty sibling rules. Preserve
all earlier failed attempts, receipts and archives in their original directories.
