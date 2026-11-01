# Current handoff

Delivery selected: Persistence 0.61.0 (Memory and Storage), Metrics Stable 0.62.0
and Logging Stable 0.61.0, supplier-first to crates.io and GitHub
`dragginzgame/ic-persistence` on `main`. Catalog minimums now require Logging
0.23.6 and Metrics 0.24.5, matching the incoming selected lock identities.
Their release notes preserve engine/arithmetic/retained-data contracts. The
incoming native Testkit 0.76.1 selection is retained, with CLI 0.76.0 separately
pinned. Full group gates and final registry/tag readback establish delivery.
Evidence is retained under `target/delivery/hard-cut-observability-2026-11-01/`.

The additional four-part public API hard cut is implemented locally: explicit
Storage `create`/`load`/`reset`, one bound-enforcing `StableCell`, validated-key
runtime lookup/open, and nonzero bucket configuration. All maintained callers,
examples, fixtures and docs use the new API. See
[the migration](../migration/0.61-public-api.md); valid stored bytes are unchanged.
The existing pending minor group notes include this batch; package versions and
publication remain unchanged. Evidence is under `target/hard-cut-api-2026-11-01/`.
Focused proof passed 72 Memory runtime tests, 20 Storage tests, both stable
adapter suites, public runtime/composition tests, capability compile-fail tests,
doctests, Clippy, native/Wasm Rust 1.91 checks and all existing Wasm budgets.
Archive-backed consumer qualification and original-Wasm installed IO, custody,
Storage/Logging reopen, retention and update/upgrade rollback passed; installed
input hashes verified unchanged under `installed-original.NMq49l/`. Full release
gates, optimizer variants and native macOS were not replayed for this API batch.
Incoming dependency lock edits remain outside the API change; no dependency
resolution or own-package release was performed.

The ten-item complexity follow-up is implemented locally for eight items; the
shared bootstrap observer and native group-policy projection are delivered for
review in [Shared Tooling PR #336](https://github.com/dragginzgame/shared-tooling/pull/336).
Their local removal awaits reviewed export/adoption; vendored files and normal
sibling checkouts remain unchanged.

Build 0.73.5 now owns archive extraction, frozen external selection, exact
extracted graph admission and final source/archive byte/mode custody. The native
consumer retains the existing library checks, native development/MSRV execution
and Wasm check roster. The old Perl manifest renderer and jq graph predicates
are removed. The incoming Build/Host 0.73.5 lock selection is preserved; the
separate Build editor remains 0.73.3.

Release checks carry inspected commit evidence within a phase and re-inspect
across packaging and remote effects. Metadata roles are classified in the plan,
including opaque extra manifests. Fixture manifest/lock inputs share one roster,
exclusive directory names use one process sequence, and installed Logging setup
reuses the existing helper. Logging overlap comparison traverses its issued
prefix once, preserving watermark and row-regression checks.

`LedgerCommitError::Codec` now carries public `LedgerDecodeError` variants for
Wire, key and slot failures, retaining typed error sources. This is a breaking
Rust error contract; durable bytes and admission remain unchanged. Pending notes
select Persistence 0.61.0, Metrics Stable 0.62.0 and Logging Stable 0.61.0 for
aligned public core identities. Own package versions remain unchanged. The core
release metadata now owns `releases/persistence/0.61.md`; published 0.60 notes
remain retained. No reset, legacy decoder or migration is introduced.

Focused proof passed 75 existing release-helper tests plus the extra-manifest
regression, 46 ledger tests, seven diagnostics tests, 12 stable-cell tests,
three Logging continuity tests and seven Logging encoding tests. Affected
Clippy, native helper/library Rust 1.91, Wasm library Rust 1.91, graph/pin/storage
boundaries, formatting, selected ShellCheck and documentation checks passed.
Actual archive-backed native/MSRV/Wasm qualification passed through the new Build
API. The consumer fixture refuses a substituted archive before compilation and
qualifies MSRV dispatch using explicit compiler substitutes.

Focused installed Logging retention, reopen and update/upgrade rollback passed
with prepared Testkit 0.76.0 and PocketIC 16.1.0, retaining evidence under
`target/qualification/runtime/attempt.rTl8f3/`. The unchanged optimizer matrix,
other installed suites, full release gates and native macOS were not replayed.
Local evidence, including earlier failed attempts, is retained under
`target/complexity-followup-2026-11-01/`.

The Shared contribution passed its focused policy and six interrupted-state
fixture, actual Cargo resolved admission for all three groups, selected ShellCheck,
links and version consistency. An isolated launcher prototype passed Persistence's
real six-state scratch bootstrap fixture with its prepared editor and unchanged
source hashes. The maintained launcher is unchanged. Hosted upstream CI remains
separate evidence; no optional matrix is awaited or implied to have passed.

The preceding Testkit/Wire delivery is published on GitHub and crates.io:
Persistence 0.60.7, Metrics Stable 0.61.5 and Logging Stable 0.60.5. Each group
passed its full gate; registry checksums and annotated remote tags verified.
Evidence remains under `target/delivery/testkit-wire-2026-11-01/`.
Platform 0.16 delivery evidence remains under
`target/delivery/platform-0.16-2026-11-01/`; earlier cleanup and dependency
proof remains in its original `target/delivery/` and audit directories.

Validation reuse remains a separate investigation in
[Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334) and
[the audit](../audits/2026-11-01-release-validation-reuse.md); no gate is skipped.
The reviewed Shared snapshot remains
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a` (recorded VERSION 0.27.21),
with 79 roots and 133 payloads. No dirty sibling rules are adopted.
