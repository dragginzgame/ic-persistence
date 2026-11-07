# Current handoff — 2026-11-07

## Pending 0.68.5: Cell header writes and Host adoption

Combine Cell header writes: Wasm −662 bytes, batch-32 handler −0.93%; other depth cells equal.
The [study](../audits/recurring/storage-write-paths-2026-11-07.md) owns 73-type/native/runtime proof.
Both B-tree trials were reverted. No version bump, commit or delivery.

Host Artifacts/FS/Process 0.75.0 and matching Build/editor 0.78.0 are adopted.
All 88 helper tests, strict lint, pins/production graphs, formatting and real
offline metadata recovery for all three groups pass; editor setup/check pass.
Evidence: `target/evidence/ic-host-075-adoption/`; independent workspace unchanged.

## Rejected terminal-iterator experiments

The [ordered-iterator study](../audits/recurring/storage-ordered-iteration-2026-11-07.md)
retains both rejected projection trials; deployed 0.68.4 entry shortcuts remain.
Raw evidence: `target/evidence/storage-projection-last/`, `storage-next-back/`.
The local 5–10 KB Wasm-growth guideline remains binding absent exceptional benefit.

## Delivered: Storage 0.68.4

Published and non-yanked at `663662663c702e2003ac88d25596f5bfd71be470`, tag
`storage-v0.68.4`. Shared batch `batch-Er9VPHO8` passed all 12 release stages,
final archive qualification, publication and guarded local reconciliation.
Downloaded archive checksum and clean VCS identity match the final receipt;
evidence: `target/evidence/storage-0684-deploy/`. Receipts remain in
`target/release-frozen/batch-Er9VPHO8/target/release-validation/`; durable recovery
is `.git/release-state/frozen/batch-Er9VPHO8/`. No cleanup or adapter release.

Prior Build proof: `target/evidence/ic-build-adoption/`; Runtime 0.19.5 original/O3/Os/Oz
installed IO, custody, upgrade and rollback: `target/qualification/runtime/attempt.uMwbLm`.
Reuse lazy-key read buffers for types with destructors; preserve fixed-key paths.
Map/Set entry iterator `last()` now uses the existing reverse cursor. Public
APIs, formats and governance are unchanged; no host migration is required.
The [backing study](../audits/recurring/storage-icss-backing-parity-2026-11-06.md)
and ordered-iterator study own the matrices. The key-buffer step improves selected string
handlers 1.5–5.2% versus 0.68.3; evidence: `target/evidence/storage-owned-key/`.
The 11,772-call matrix improves all 90 entry-last cells; the ordered study retains
handler gains, a 2.24% worst regression and +296,606-byte benchmark Wasm growth.
All 23 adapter and 49 benchmark tests and focused installed qualification passed.
Evidence was recorded under `target/evidence/storage-iterator-last/`; full type
timing was not rerun. Release qualification is separate from benchmark claims.

## Delivered: Storage 0.68.3

Published and non-yanked at `50dbd7f8b8fbc1516ccf9630a0e26bc3539a5b90`,
with annotated tag `storage-v0.68.3`. Shared batch `batch-RAD7_qIV` passed all
12 release stages, final archive qualification, publication and guarded local
reconciliation. Registry checksum, downloaded archive and clean VCS identity
match the final receipt; evidence: `target/evidence/storage-0683-deploy/`.
Receipts remain under `target/release-frozen/batch-RAD7_qIV/target/release-validation/`;
durable recovery remains `.git/release-state/frozen/batch-RAD7_qIV/`. No cleanup.

Routing-read changes retain APIs and layouts; the backing study owns gains,
scan tradeoffs and the 160-case native IO proof. Evidence was recorded under
`target/evidence/storage-map-routing/`, with depth runs `qKBykW`/`SpnBn9`/`LLDlVi`.
Original/O3/Os/Oz installed qualification passed in `attempt.UjEWo6` and
`attempt.F3ZzEd` under `target/qualification/runtime/`.
Build 0.76.0 and Wire 0.17.6 adoption evidence was recorded under
`target/evidence/build-076-wire-0176/`, including the initial CLI-admission failure.

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
Both locks select Runtime 0.19.5; root Wire is now 0.17.7 (independent edit), benchmark 0.17.6.
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

The authorized IcyDB layout preparation exists only in `/tmp/icydb-control-layout`,
branch `storage/control-allocation`, base `03c5713e`. Preserve its complete
uncommitted tracked/untracked work; do not clean or continue implementation there.
Its 685 focused tests and native/Wasm lint passed; installed composition is unqualified.
Approval is pending for durable retention at `/home/adam/projects/icydb-control-layout`
or an authorized draft PR. Do not modify the sibling or create an unsolicited PR.
