# Current handoff — 2026-11-07

## Pending routing-read improvements

Read B-tree child addresses into their final vector; batch already-eager fixed
keys up to 16 bytes. Map, Set and regions keep their APIs, formats and governance.
Against published ICSS 0.7.2, the selected 4,096-record full-handler matrix is
9.7% cheaper for Map Get, 5.2% for Replace, 13.9% for Set Get and 6.1% for region
Get; region Replace is effectively equal. Baseline Map Get was already at parity
outside the inner timer. These are synthetic results, not complete endpoints.

The [backing study](../audits/recurring/storage-icss-backing-parity-2026-11-06.md)
owns both scopes and trial matrices. Raw evidence: `target/evidence/storage-map-routing/`;
ordinary depth runs: `qKBykW`, `SpnBn9`, `LLDlVi`. Each checks 3,936 calls;
round-trip and boundary-probe inner counts match. The wider ordered matrix checks
10,476 calls per run: 1,654/1,746 full-handler cells improve; worst regression is
1.63% for a 128-entry string scan (0.75% above ICSS). Wide scans remain a tradeoff.
Native IO checks 160 matched cases: fewer calls with identical bytes/codecs/ranges.
Focused engine, collection, driver and ICSS image-interchange tests pass, as do
strict Clippy and original/O3/Os/Oz installed IO, custody, upgrade and rollback
(`target/qualification/runtime/attempt.UjEWo6`). No source commit or deployment.
Compatible Storage 0.68.3 notes include this work and the preserved adoption below.

## Pending IC Build and IC Wire adoption

Select published IC Build 0.76.0 for the native helper and release-metadata CLI;
both the library requirement and explicit CLI selection are aligned. Storage has
no callers of Build's changed cache-batch/test-support APIs. The selected CLI is
prepared and admitted. Resolve published IC Wire 0.17.6 in both maintained locks;
the incoming root already selected it and is preserved. Keep Wire's compatible
0.17.0 public requirement: none of the new opt-in field adapters is required or
selected. Collection, directory and adapter checkpoint codecs are unchanged.

Both locked graphs pass destination/singleton admission and selected native/Wasm
Clippy. All 114 library and 88 helper tests, strict helper Clippy, bootstrap
recovery, pin/boundary checks and formatting pass. Installed original/O3/Os/Oz
IO, custody, collection reopening, upgrade, rollback and Logging checkpoint
qualification passes in `target/qualification/runtime/attempt.F3ZzEd`. Testkit
0.79.0 was re-prepared for changed native build inputs; the 16.1.0 server selection
is unchanged. Evidence: `target/evidence/build-076-wire-0176/`. The first installed
attempt refused CLI admission before execution; its log remains retained.
Package versions are unchanged, with Storage 0.68.3 notes prepared. No source
commit, release, push or publication is included in this adoption request.

## Delivered: Storage 0.68.2 through Shared release batches

Storage 0.68.2 is published and non-yanked, with annotated tag `storage-v0.68.2`
at `1c08ecf6ec8c0aa01c9c9fe73f5d2d818df2d9dc`. Shared 0.31.0 (`7431394`)
captured `f911ba0` in `batch-1QTqeCag`; all 12 stages, publication and guarded
reconciliation passed. The normal branch fast-forwarded; adapters did not change.

The downloaded registry archive and sparse index match the final receipt checksum
`accdfc2a7880dcdd5ce38495a71fc19e70c54d7546ce1cea47adeb7858745c2b`;
its clean embedded VCS identity matches the independently observed GitHub tag.
Receipt/logs remain under `target/release-frozen/batch-1QTqeCag/target/`;
durable intent/Git state is `.git/release-state/frozen/batch-1QTqeCag/`.
Verification, status and reconciliation evidence: `target/evidence/storage-0682-deploy/`.
No batch or old evidence was retired. Hosted CI remains a separate observation.

Failed `batch-DqMimxOk` remains intact: inherited publication context broke a fixture
before version/Git/registry effects. The corrected fixture passed in the new batch.
Initial adoption evidence: `target/evidence/shared-tooling-0310/`. Delivery also
qualified the existing Wire 0.17.5 lock selection.

## Earlier delivered groups

Storage 0.68.1 is published at `6ee75e906987fe20f73bb837c9107d9b5312da7e`,
with tag `storage-v0.68.1`; archive checksum/VCS readback and all 12 stages passed.
It adopted published Runtime 0.19.4 and shipped the earlier Vector/backing work.
Evidence: `target/evidence/storage-0681-deploy/`; installed `attempt.XawP1b`.

The prior coordinated hard cut published Storage 0.68.0 (`6f87f36`), Metrics
Storage 0.70.0 (`388c366`) and Logging Storage 0.66.0 (`7ac245c`), with their
group tags, all 12 stages and independently verified archives. Receipts remain
under `target/release-validation/`; evidence: `target/evidence/storage-068-deploy/`.
Installed proof: `attempt.P0oWYq`. Neither adapter has been republished since.
No publication needs replaying. Full history remains in the owning release notes.

## Breaking collection baseline

Storage 0.68 owns private copied ICSS 0.7.2 engines behind governed adapters;
Map/Set/regions share one B-tree, Vector uses fixed-stride slots and Cell caches
its value. No production ICSS dependency or copied MemoryManager exists.
Old independent engines, collection checksums, node cache and Vector windows/
epochs are retired. [Provenance](../storage/upstreams.md) retains notices,
source identities and the deliberate safety/integration adaptations.

Existing collection images require an explicit host rebuild/reset or fresh
admitted allocation. BTR2 with explicit page size, SVC1 and SCL1 are current;
there is no legacy reader, migration or automatic reset. Directory encoding and
application codecs are unchanged. Engine/codec failures trap; framing admission
and reservation retain typed errors. Native mutation unwinds poison the owner;
Wasm relies on IC rollback. Positive cache ceilings do not enable a node cache.

Metrics Storage 0.70 and Logging Storage 0.66 align public Storage handles/errors
with 0.68. Metric snapshot and discriminator-3 logging payload semantics remain
unchanged, but their underlying collection images require the same host action.
See [Storage](../../releases/storage/0.68.md),
[Metrics Storage](../../releases/metrics-storage/0.70.md) and
[Logging Storage](../../releases/logging-storage/0.66.md) release notes.

## Current tooling and comparison evidence

Shared Tooling 0.31.0 is adopted; its live Storage batch proof is above.
[Adoption boundaries](../shared-tooling-adoption.md) retain earlier tooling proofs.
Both locks select Runtime 0.19.4 and Wire 0.17.6; Build 0.76.0 is pending above.
Native production excludes Runtime/Types; Wasm selects Runtime IC0 only.

The [type catalogue](../audits/recurring/benchmark-full-catalogue-2026-11-05.md)
retains 73 types / 565 cases across five forms, 53 verified refusals and 18
non-Ord roles. Its earlier 46,604-call results are historical, not new timings.
All comparisons use an eight-MiB synthetic Wasm stack; production is unchanged.
Canic configuration preparation and Auth inner decoding remain outside scope.

## Historical evidence and preservation

The [measurement index](../measurements/README.md) owns compact evidence and its
retention limits. The authorized 2026-11-06 cleanup retired bulk/repeated output;
raw target artifacts remain transient and removable by explicit cargo clean.
No cache, stash, target or release/recovery evidence was removed in this pass.
CI group evidence remains in `target/ci-group-selection/` and
[the validation mapping](../storage-release-validation.md).

Preserve `storage-tooling-030-preserved-pending-lock-inputs`,
`storage-0670-preserved-pending-lock-inputs`, all `metrics-068-*` stashes,
`storage-0660-unselected-lock-edits`, `storage-0660-preserved-pending-inputs`,
`storage-0655-unselected-lock-edits`, `storage-0654-unselected-lock-edits`,
`storage-0653-unselected-lock-edits`, `storage-0653-hotspot-integration`, all other
preservation stashes, failed attempts, historical caches, receipts and archives.
Closed #153/#154 and publication evidence remain linked by the delivered handoff.

## Deferred work and boundaries

The [consumer inventory](../local-memory-audit.md) lists declarations, not live
usage. Canic receipt bounds and complete Auth endpoints remain unqualified.
Any adoption must explicitly replace old collection images and host framing.
No current public API raw-backing extraction or independent ID assignment is added.

The authorized IcyDB layout preparation exists only in `/tmp/icydb-control-layout`,
branch `storage/control-allocation`, base `03c5713e`. Preserve its complete
uncommitted tracked/untracked work; do not clean or continue implementation there.
Its 685 focused tests and native/Wasm lint passed; installed composition is unqualified.
Approval is pending for durable retention at `/home/adam/projects/icydb-control-layout`
or an authorized draft PR. Do not modify the sibling or create an unsolicited PR.

Native production has no Runtime/Types. Wasm selects Runtime IC0 only, without
executor, CDK, guard, Types or Candid. Independent qualification uses Platform/
PocketIC 0.19.0, Testkit 0.79.0 and server 16.1.0. Platform 0.17 remains only as
the explicit ICSS 0.7.2 benchmark facade; pinned Git `b5d1650` is a research control.
