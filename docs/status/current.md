# Current handoff

Released source is 0.54.0 at `9ca2e8e39f8ab5d1b8501e4d843b25c614719a13`.
Pending 0.54.1 adopts Testkit/host tooling 0.69.11 and adds an unpublished Logging
Stable workspace scaffold. Package metadata remains 0.54.0; no release,
publication, staging or push occurred. See [pending notes](../changelog/0.54.md#0541---unreleased).

## Current development batch

- Align Build, artifact/filesystem/process helpers and native Testkit requirements
  to 0.69.11, plus the separate Build and Testkit CLI selections. Reviewed upstream
  release commit is `d7aed4d0f878a8e8c1686fc71e2bbe45f1062f82`.
- Tooling adoption preserved incoming selections. The logging scaffold adds one
  local lock record and no dependencies. The library lock already selected host
  tooling 0.69.11 and Platform 0.7.5. Both Platform declarations
  remain 0.7.3 and the independent qualification lock retains Platform 0.7.3.
- Shared Tooling stays at 0.27.6, commit
  `5a72359b5be19d5221334cf2c9b3204621d3f3f6`. Metrics stays at 0.17.0.
  Features, Rust support, stable-structures 0.7.2 and PocketIC 16.1.0 remain.
- The upstream Testkit snapshot/process cleanup requires no Persistence caller
  migration. Public APIs, V1 records, collection formats and custody are unchanged.
  Scheduled maintenance remains inactive.
- `ic-logging-stable` has inherited metadata, an MIT license and a handoff guide,
  but no runtime API, stored format or upstream dependency. The maintainer owns
  the Observability restoration work. Version projection and bootstrap cover all
  four members; publication and receipts still cover the existing three crates.

## Focused evidence and installation

Inputs are retained at `/tmp/ic-persistence-tooling-06911.OMkw5Z/`, including both incoming
locks and the preceding handoff. Formatting, snapshot integrity, dependency pins
and the Wasm production graph pass. All 65 native release-helper tests and strict
release-helper Clippy pass with host tooling 0.69.11.

The logging scaffold compiles natively and for Wasm. Metadata confirms publication
is disabled and there are no dependencies. Pin/graph, license and documentation
checks pass. Updated 65-case release-helper and consumer adapter fixtures pass.
Actual offline bootstrap with all four members admits both interrupted metadata
states without changing fixture inputs; retained evidence is at
`/tmp/ic-persistence-bootstrap.CzvHVp/`. Adapter evidence is at
`/tmp/ic-memory-release-adapters.CsRr2e/`; the earlier graph-stub mismatch is
retained at `/tmp/ic-memory-release-adapters.B6xz1e/`.

Build and Testkit CLI 0.69.11 setup pass. The first Testkit installation wait
expired while ic-timers owned the shared slot. A later setup observed its release
and reused the completed installation without modifying the foreign lock/build.
Subsequent offline Build CLI and Testkit CLI/server admission both pass.
Full release and installed-runtime qualification were not run for this adoption;
prior native/installed evidence is not relabelled as 0.69.11 qualification.

## Preserved runtime and recovery evidence

The 0.53.1 fix retires backing identity claims before freeing their allocations,
including failed construction and unwinding. Duplicate-owner refusal, detached
lease custody and the 0.53.0 checkpoint-validation contract remain in force.
Construction, initial bootstrap, new leases, diagnostics and growth check full
metadata. Ordinary size/IO trust exclusive custody and reject poisoned owners;
raw aliases and unmanaged managers remain prohibited. No hot-path scan or new
unchecked mode is introduced.

The 0.54.0 adoption evidence remains at `/tmp/ic-persistence-adoption-20261029/`.
Earlier runtime evidence remains at `/tmp/ic-persistence-teardown-137/`,
`/tmp/ic-persistence-cost-137/`, `/tmp/ic-persistence-issue-137/` and
`/tmp/ic-persistence-fixture-custody-fix/`. Their original source identities and
native qualification limits remain with those artifacts and retained handoffs.
Preserve all release journals, failed attempts, receipts and retained archives.
Keep retirement backups at `/home/adam/projects/ic-persistence-retirement-2026-10-20/`
and historical exports linked by the migration guides. No repository deletion
or retirement retry is in scope.
