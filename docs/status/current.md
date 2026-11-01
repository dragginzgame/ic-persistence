# Current handoff

Delivery selected: compatible Persistence 0.61.1, Metrics Stable 0.62.1 and
Logging Stable 0.61.1 to crates.io and GitHub `dragginzgame/ic-persistence`, `main`.
The standard group gates, final archive receipts and registry/tag readback own
delivery evidence under `target/delivery/platform-wire-2026-11-01/`. Earlier
unpublished wording below describes preparation before this delivery selection.

The normal checkout is synchronized with remote main at `9864637`, including
published Persistence 0.61.0 (Memory/Storage), Metrics Stable 0.62.0 and Logging
Stable 0.61.0, plus merged PRs #147–150. The releases require Logging 0.23.6 and
Metrics 0.24.5. All three full gates, final archive qualification, crates.io
checksums and annotated remote tags passed. Delivery evidence remains under
`target/delivery/hard-cut-observability-2026-11-01/`.

Those releases used its isolated `checkout/` to preserve incoming lockfile work.
The original failed Persistence plan and its passing pre-bump gate remain as
failure evidence; do not resume that superseded attempt. The completed isolated
plans and final registry readback establish delivery. Normal checkout fast-forward
used Git autostash; the incoming dependency patch and original lock were retained
under `target/wire-0.13.5-adoption/` before reapplication.

Current adoption selects published Wire/Derive 0.13.5 in both maintained lockfiles.
The compatible 0.13.3 catalog requirements remain sufficient. Supplier encoding
optimizations preserve bytes, budgets, primitive acceptance and error behavior;
no migration or host reset is required. Incoming root Build/Host 0.73.6 and Testkit
Native 0.76.2 selections are preserved. The separate Build editor remains 0.73.3,
Testkit CLI 0.76.0 and PocketIC 16.1.0. No executable pins are changed.

Pending compatible notes select Persistence 0.61.1, Metrics Stable 0.62.1 and
Logging Stable 0.61.1; package release versions remain at the published values.
This Wire adoption is local and unpublished. Both locked graphs, singleton policy,
selected Wasm/native Clippy and pin checks passed. Encoding fixtures passed 13
ledger, seven Logging and 13 Metrics tests; one existing ledger probe stays ignored.
Affected Wasm libraries passed Rust 1.91. Documentation links and diff checks passed.
Evidence is retained under `target/wire-0.13.5-adoption/`; no new full release gate
or installed runtime qualification is implied by dependency admission.

Platform 0.17.0 is now selected in both catalogs and lockfiles. The stable-structures
facade and substrate 0.7.2 are unchanged. The native qualifier uses owned
`ic-platform-pocket-ic` 0.17.0 with server 16.1.0; Logging's qualification fixture
uses Runtime/macros/guard 0.17.1 and Types 0.16.0. Testkit's existing Platform 0.16
dependency remains confined to native measurement helpers; production retains one
provider. No public persistence API, retained format or executable pin changed.

Combined Platform/Wire destination metadata, singleton policy, Wasm/native Clippy,
production graph and pin checks passed. All three original Wasm fixtures and the
native qualifier built on Rust 1.91. Memory runtime (72) and Storage (20) tests
passed. Installed Linux IO, Storage, custody, Logging reopen/retention and
update/upgrade rollback passed with the owned client; post-run input hashes
verified unchanged. Evidence is retained under `target/platform-0.17-adoption/`.
Optimizer variants, native macOS and full release gates were not replayed.
This compatible adoption shares the existing pending patch notes and is unpublished.

The published API hard cut uses explicit Storage `create`/`load`/`reset`, one
bound-enforcing `StableCell`, parsed `StableKey` runtime lookup/open and nonzero
bucket configuration. `LedgerCommitError::Codec` retains typed
`LedgerDecodeError` sources. See [the migration](../migration/0.61-public-api.md).
Stored formats and allocation custody remain unchanged. Earlier focused and
installed IO/upgrade/rollback proof is retained under `target/hard-cut-api-2026-11-01/`
and `target/complexity-followup-2026-11-01/`.

Build owns archive extraction, exact extracted graph admission, frozen external
selection and final input custody. The consumer owns the library/native/MSRV/Wasm
roster. Shared's grouped source-only bootstrap observer is adopted from
[Shared PR #336](https://github.com/dragginzgame/shared-tooling/pull/336); launcher
metadata rechecks and scratch recovery remain in place.

The merged destination-adoption check admits both independent locked graphs
before selected consumer compilation. See
[Shared #335](https://github.com/dragginzgame/shared-tooling/issues/335).
Opt-in exact-input ShellCheck reuse retains source/index admission, original
execution evidence and separate current group admissions; complete gates remain
unchanged. The real stage executed once and reused twice locally across group
projections. Native CI remains separate acceptance for
[Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334).
See [adoption](../shared-tooling-adoption.md#exact-input-shellcheck-reuse) and
[the investigation](../audits/2026-11-01-release-validation-reuse.md).

PR #150 raises the caller input limit to 64 MiB for native macOS ShellCheck and
prints failed group logs while preserving their status. Earlier failed native
qualification remains retained in GitHub run 41314835837; corrected native
acceptance is separate from the local proof.

The reviewed Shared snapshot remains
`9275bf0de39ed941d81c47bab97eef7f6fc38df2` (VERSION 0.27.24), with 81 roots and
136 payloads. Never patch vendored files or adopt dirty sibling rules. Earlier
Testkit/Wire and Platform delivery evidence remains in its original `target/delivery/`
directories; retain failed attempts, receipts and archives.
