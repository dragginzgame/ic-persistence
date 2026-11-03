# Current handoff

## Metrics 0.28 and Wire 0.16 delivery preparation

Published Metrics **0.28.0** is selected in the root catalog/lockfile. It has no
Wire dependency or native schema feature; Storage still owns its record codec.
Both maintained catalogs/lockfiles select Wire/Derive **0.16.0**. Public metric
values/errors and Wire errors/limits change Rust identity; retained rows and
current Storage-owned backing do not require reset or migration.

The requested release scope is Storage **0.66.0**, Metrics Storage **0.67.0**,
then Logging Storage **0.64.0**, supplier first, from the normal
`/home/adam/projects/ic-storage` checkout to the existing crates.io identities.
Direct `origin/main` push requires the separately disclosed explicit approval.
No new release or publication has run at this preparation stage.

[Storage #154](https://github.com/dragginzgame/ic-storage/issues/154) has its local
schema-coupling fix merged in [PR #155](https://github.com/dragginzgame/ic-storage/pull/155),
remote main `f280cb8`. The new supplier completes its previously unavailable
acceptance input. All 29 Metrics tests, strict native/Wasm Clippy, production
graphs, pins, formatting and snapshot integrity pass with Metrics 0.28.0.
Evidence: `target/metrics-028-deploy/`. Close the issue after confirming the
qualified adoption is delivered on main. Observability #128 remains supplier-owned.

The pending batch includes reviewed Shared Tooling **0.27.29** (`18e7270`),
exported from clean committed source. Its 145-file snapshot, three native group
plans, release-adapter substitutes and affected documentation passed earlier
qualification in `target/shared-tooling-02729-adoption/`. Tool pins and Rust/Cargo
1.99.1 are unchanged. Earlier Wire 0.16 focused evidence (129 selected tests and
seven native/Wasm graphs) remains in `target/wire-016-adoption/`; release gates
and installed qualification must use the final source/selected dependency graph.

Unrelated Build 0.74.1, Host 0.74.2 and Logging 0.24.3 lock selections are preserved
in Git stash `storage-0660-preserved-pending-inputs` and the incoming lock evidence.
They are excluded from this release source and must be restored afterward without
changing the released workspace versions or Metrics/Wire selections. Never blindly
apply the whole stash: it also contains already-selected adoption changes.

## Prior publication and recovery

[Storage #153](https://github.com/dragginzgame/ic-storage/issues/153) is closed on
published Storage **0.65.5**, commit `127031ee`, tag `storage-v0.65.5`, and IcyDB's
registry consumer evidence. Its verified archive SHA-256 is
`4a5b1ea9a7b8f61e0ea22b257be5282795bd24ba77ab3d730afd02a46462b9aa`.
Prior full gate, archives and original/O3/Os/Oz installed IO/upgrade/rollback proof
remain in `target/storage-0655-deploy/`; do not repeat that release/upload.
[Its delivered handoff](https://github.com/dragginzgame/ic-storage/blob/ce52130d9e945aefee017faf905fc65e1db40cb3/docs/status/current.md)
retains exact earlier qualification paths and benchmark interpretations.

Keep the `storage-0655-unselected-lock-edits`, `storage-0654-unselected-lock-edits`,
`storage-0653-unselected-lock-edits` and `storage-0653-hotspot-integration` stashes.
Earlier gate failures involved external lock edits and shared-target trybuild path
mismatches; their logs and final successful independent qualification remain intact.
No historical tool caches, recovery receipts or archives are cleanup candidates.

## Deferred memory and performance work

The [consumer audit](../local-memory-audit.md) records source inventories, not live
bucket usage: Canic 27 base plus five optional allocations, Blob 14, Relay three
per channel plus component stores, IcyDB `3 + 4J`. Canic receipt bounds still need
qualification against Storage; no capacity failure or quantified saving is claimed.

The explicitly authorized IcyDB layout preparation exists only in
`/tmp/icydb-control-layout`, branch `storage/control-allocation`, base `03c5713e`.
It is uncommitted: preserve it, do not clean or continue implementation there.
It separates schema controls, admits fresh images and rejects incompatible ones;
685 focused tests and native/Wasm lint passed, but installed composition is unqualified.
Approval remains pending to retain its complete tracked/untracked changes at
`/home/adam/projects/icydb-control-layout` or prepare an authorized draft PR there.
Do not change the normal sibling checkout or create an unsolicited PR.

The prior node-buffer study shows small integer-path improvements but a **12.74%
4 KiB batch-scan regression**, with total 4 KiB trace +0.15%; cause is unverified.
Cell checksum throughput and external-payload scans remain profiling candidates.
Matched measurements remain in `target/storage-node-buffers/` and
`target/storage-comparison/`; nine matched 1/8/128-page ICSS bucket comparisons
remain in `target/storage-bucket-study/`. These are collection-level measurements,
not whole-endpoint speedups or a universal Storage/ICSS ranking.

## Retained product boundaries

Storage owns allocation, custody and five collection forms; opaque IDs, host
lifecycle/framing and explicit fresh/reset backing remain. Map/Set/regions retain
`ICMAP002`; retired manager/collection images require explicit rebuild, never
an automatic reset or legacy decoder. Native failures poison ownership; IC traps
use rollback. Current rows are unchanged by this Metrics/Wire adoption.

Native production has no Runtime/Types. Default Wasm uses Runtime 0.19 IC0 only,
without executor, CDK, guard, Types or Candid. The independent qualifier uses
Platform/PocketIC 0.19.0, Testkit 0.79.0 and server 16.1.0. Platform 0.17 remains
only as the explicit ICSS 0.7.2 benchmark facade; pinned DFINITY Git `b5d1650` is
another research control. No sibling release or backing reset is authorized here.
