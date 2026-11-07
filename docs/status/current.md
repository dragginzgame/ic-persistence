# Current handoff — 2026-11-07

## Pending 0.68.4: ordered reads and IC Build

IC Build 0.77.0 is selected for the native helper and prepared release editor.
Storage has no callers of its removed cache-timing getters. All 88 helper tests,
strict lint, native/Wasm graphs, pins and three-group interrupted metadata
recovery pass; evidence: `target/evidence/ic-build-adoption/`. Both lockfiles now
select Runtime 0.19.5; original/O3/Os/Oz installed IO, custody, upgrade and rollback
pass in `target/qualification/runtime/attempt.uMwbLm` for deployment.

Reuse lazy-key read buffers for types with destructors; preserve fixed-key paths.
Map/Set entry iterator `last()` now uses the existing reverse cursor. Public
APIs, formats and governance are unchanged; no version, commit or publication.
The [backing study](../audits/recurring/storage-icss-backing-parity-2026-11-06.md)
owns both steps' matrices. The earlier key-buffer step improves selected string
handlers 1.5–5.2% versus 0.68.3; evidence: `target/evidence/storage-owned-key/`.
The new 11,772-call ordered matrix improves all 90 iterator-last cells. Selected
32-query handlers improve 85–94% against the pending baseline and 86–95% against
ICSS 0.7.2. Worst other handler: +2.24%; full benchmark Wasm: +296,606 bytes.
Keys/values projections remain unchanged. All 23 adapter and 49 benchmark tests,
strict lint and focused installed Storage upgrade/rollback pass. Frozen boundary
probes preserve all loop counts. Evidence: `target/evidence/storage-iterator-last/`.
Full type timing and release gates remain separate from these performance proofs.

## Delivered: Storage 0.68.3

Published and non-yanked at `50dbd7f8b8fbc1516ccf9630a0e26bc3539a5b90`,
with annotated tag `storage-v0.68.3`. Shared batch `batch-RAD7_qIV` passed all
12 release stages, final archive qualification, publication and guarded local
reconciliation. Registry checksum, downloaded archive and clean VCS identity
match the final receipt; evidence: `target/evidence/storage-0683-deploy/`.
Receipts remain under `target/release-frozen/batch-RAD7_qIV/target/release-validation/`;
durable recovery remains `.git/release-state/frozen/batch-RAD7_qIV/`. No cleanup.

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
(`target/qualification/runtime/attempt.UjEWo6`). No format migration is required.

Storage 0.68.3 adopted Build 0.76.0 and Wire 0.17.6. Wire's compatible 0.17.0
minimum and all stored codecs remained unchanged; details are in its release notes.

Dependency checks and installed original/O3/Os/Oz qualification passed in
`target/qualification/runtime/attempt.F3ZzEd`; evidence: `target/evidence/build-076-wire-0176/`.
Testkit 0.79.0 was re-prepared after native build-input changes; server 16.1.0
is unchanged. The initial CLI-admission failure remains retained.

## Earlier delivered groups and retained recovery

Storage 0.68.2 (`1c08ecf`, tag `storage-v0.68.2`) passed all 12 stages, registry
verification and reconciliation in `batch-1QTqeCag`; adapters did not change.
Evidence: `target/evidence/storage-0682-deploy/`. Captured/durable batch state
remains under `target/release-frozen/` and `.git/release-state/frozen/`.
Failed `batch-DqMimxOk` remains intact: inherited publication context broke a fixture
before version/Git/registry effects. The corrected fixture passed in the new batch.
Initial adoption evidence: `target/evidence/shared-tooling-0310/`. Delivery also
qualified the existing Wire 0.17.5 lock selection.

Storage 0.68.1 is published at `6ee75e906987fe20f73bb837c9107d9b5312da7e`,
with tag `storage-v0.68.1`; archive checksum/VCS readback and all 12 stages passed.
It adopted published Runtime 0.19.4 and shipped the earlier Vector/backing work.
Evidence: `target/evidence/storage-0681-deploy/`; installed `attempt.XawP1b`.

The prior coordinated hard cut published Storage 0.68.0 (`6f87f36`), Metrics
Storage 0.70.0 (`388c366`) and Logging Storage 0.66.0 (`7ac245c`), with their
group tags, all 12 stages and independently verified archives. Receipts remain
under `target/release-validation/`; evidence: `target/evidence/storage-068-deploy/`.
Installed proof: `attempt.P0oWYq`. Neither adapter has been republished since.
No publication needs replaying; history remains in owning release notes. Hosted CI is separate.

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
Both locks select Wire 0.17.6 and Runtime 0.19.5.
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
