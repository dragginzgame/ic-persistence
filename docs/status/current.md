# Current handoff

Released source is 0.53.1 at `1583819343fda5c31bbe62bf058e1b2927189262`.
The pending release is 0.54.0 because Metrics Stable now exposes IC Metrics 0.17
arithmetic types. Hosts passing those values must select `ic-metrics = "0.17"`.
Package metadata remains 0.53.1; no release, publication, staging or push occurred.
See [pending notes](../changelog/0.54.md#0540---unreleased).

## Current development batch

- Adopt Shared Tooling 0.27.6 at
  `5a72359b5be19d5221334cf2c9b3204621d3f3f6` through its canonical exporter.
  Impact preview and independent source verification cover the existing 123
  payloads. Scheduled maintenance remains inactive.
- Select Metrics 0.17.0 and host tooling 0.69.9, including Build/Testkit CLI pins.
  Metrics now includes Serde/native Wire support; recording stays disabled.
  The root lock retains its incoming Platform 0.7.4 selection. Both Platform
  declarations remain 0.7.3 and the independent lock stays unchanged at 0.7.3.
- Centralize publication identity guards at the existing fresh checkpoints and
  share installed qualification connection/funding/installation setup.
  Scenario assertions, refusal messages and caller-owned server lifecycle remain.
- Preserve stable-structures 0.7.2, the PocketIC 16.1.0 client/server pair, feature
  selections, Rust support, V1 records, collection formats and allocation custody.

## Focused evidence

Adoption inputs and Shared impact/refresh evidence are retained at
`/tmp/ic-persistence-adoption-20261029/`, including the incoming patch, both
incoming locks and the prior handoff. Earlier cleanup edits remain in this batch.

Dependency pins, the Wasm production graph, formatting and independent snapshot
source verification pass. All 28 Metrics adapter tests, three doctests and 65
release-helper tests pass with the selected dependencies. Strict native and Wasm
Metrics Clippy and native release-helper Clippy pass. Build/Testkit 0.69.9 CLI
setup and subsequent offline admission pass using the retained 16.1.0 server.
The Metrics adapter also checks successfully on the actual Rust 1.91 minimum.
Full release and installed-runtime qualification were not run for this adoption;
prior Linux/macOS acceptance is not relabelled as new-version qualification.

## Preserved runtime and recovery evidence

The 0.53.1 fix retires backing identity claims before freeing their allocations,
including failed construction and unwinding. Duplicate-owner refusal, detached
lease custody and the 0.53.0 checkpoint-validation contract remain in force.
Construction, initial bootstrap, new leases, diagnostics and growth check full
metadata. Ordinary size/IO trust exclusive custody and reject poisoned owners;
raw aliases and unmanaged managers remain prohibited. No hot-path scan or new
unchecked mode is introduced.

Earlier evidence remains at `/tmp/ic-persistence-teardown-137/`,
`/tmp/ic-persistence-cost-137/`, `/tmp/ic-persistence-issue-137/` and
`/tmp/ic-persistence-fixture-custody-fix/`. The previous handoff retained with this
adoption records their exact source identities and native qualification limits.
Preserve all release journals, failed attempts, receipts and retained archives.
Keep retirement backups at `/home/adam/projects/ic-persistence-retirement-2026-10-20/`
and the historical exports linked by the migration guides. No repository deletion
or retirement retry is in scope.
