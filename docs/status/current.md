# Current handoff — 2026-11-06

## Local Shared Tooling refresh after publication

Shared Tooling 0.31.0 (`7431394`, 159 files) is adopted locally. Grouped releases
capture automatically; the candidate setup hook owns initial tool/cache preparation
and offline tool admission during recovery. Publication remains explicit. The old
opt-in names and runner are retired. Current instructions are in RELEASING.md
and the shared batch guide. Consumer routing/setup tests use substituted release
effects; no live batch delivery is claimed by adoption. Both incoming lockfiles
are preserved. Evidence: `target/evidence/shared-tooling-0310/`. Published Storage
0.68.1 remains unchanged; the next pending tooling notes are under 0.68.2.

## Delivered: Storage 0.68.1

This checkout owns Storage delivery; Platform is released from its own repository.
Published Runtime 0.19.4 is adopted in both maintained workspaces and is the new
minimum for the safe uninitialized stable-read export. No local Cargo override
is selected. Destination graph checks, selected native/Wasm Clippy and installed
original/O3/Os/Oz IO, custody, collection reopening, upgrade and trap rollback
pass (`target/qualification/runtime/attempt.XawP1b`). Evidence is under
`target/evidence/storage-0681-deploy/`. The earlier local-package proof remains
under `target/evidence/runtime-local-4547033/`; it is distinct from registry proof.

Storage 0.68.1 is published, non-yanked, and independently downloaded. Its archive
checksum and clean embedded VCS commit match the qualified receipt and pushed
annotated tag `storage-v0.68.1` at `6ee75e906987fe20f73bb837c9107d9b5312da7e`.
All 12 required stages passed on Rust/Cargo 1.99.1, including native tests,
tooling/recovery, rustdoc, Wasm, size budgets and all-three-package archive
integration. Receipt: `target/release-validation/storage-0.68.1.json`.
Verification: `target/evidence/storage-0681-deploy/verified.json`. The crates.io
API returned 403; independent sparse-index and static-archive verification passed.
Hosted CI remains separate from the completed local release qualification.

The released compatible changes optimize Vector last(), fixed-width direct reads,
Map/region header admission and governed backing access. The
[follow-through matrix](../audits/recurring/storage-068-followup-2026-11-06.md)
and [backing matrix](../audits/recurring/storage-icss-backing-parity-2026-11-06.md)
retain before/after comparisons against published ICSS 0.7.2. Fixed-width scalar
and 32-byte batches save 6,400 instructions, including at the full synthetic
handler boundary. The apparent older eight-page bucket gain was a timer-boundary
shift and is not credited as a handler speedup. Rejected tuning remains rejected.
Layouts and successful-call APIs are preserved; no backing migration is needed.

Shared Tooling 0.30.7 and native IC Build 0.75.0 are included. Existing root
Logging 0.24.10, Metrics 0.29.5 and Wire 0.17.4 selections are preserved and
qualified as part of this release graph. Neither adapter has an independent
source change requiring a release. Metrics Storage remains 0.70.0 and Logging
Storage remains 0.66.0; their compatible requirements admit Storage 0.68.1.

## Previous delivery: Storage 0.68.0 and both adapters

The authorized Storage-only delivery is complete on `origin/main`. All three
annotated release tags are pushed and all packages are published, non-yanked,
with independently downloaded archives matching qualified checksums and VCS IDs.
Sibling repositories were inspected only and remain unchanged by this delivery.

| Package | Version | Release commit | Tag |
| --- | --- | --- | --- |
| ic-storage | 0.68.0 | `6f87f36` | `storage-v0.68.0` |
| ic-metrics-storage | 0.70.0 | `388c366` | `metrics-storage-v0.70.0` |
| ic-logging-storage | 0.66.0 | `7ac245c` | `logging-storage-v0.66.0` |

All 12 required release stages passed for each group on Rust/Cargo 1.99.1,
including lint, features, policy, tooling/recovery, all collection/adapter tests,
rustdoc, Wasm tests, size budgets and complete archive integration. Each group's
metadata, exact source/index/tag and atomic push checks passed. Live publication
and independent readback are separate completed observations.

Receipts: `target/release-validation/storage-0.68.0.json`,
`metrics-storage-0.70.0.json` and `logging-storage-0.66.0.json` in that directory.
Release, publication and independent registry readback evidence is retained under
`target/evidence/storage-068-deploy/`. Earlier failed gate logs remain untouched:
the Candid roster expectation and native-only Vector test scope were corrected,
then the complete gate passed. There is no pending publication to replay.

Installed original/O3/Os/Oz qualification passed for IO, backing custody,
collections and Logging, including 1,031 allocations, upgrade reopening and
trapped decoder/update/upgrade rollback. Exact inputs and all four variants are
in `target/qualification/runtime/attempt.P0oWYq/`. No deployed canister backing
was reset and no sibling adoption is implied.

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

Shared Tooling 0.30.7 (`22a853e`, 159 files) is adopted through the canonical
exporter. Candid/CBOR lexical checking covers all 164 first-party Rust files;
both workspaces reject named direct CBOR dependencies. Root admission uses the
shared inventory; independent qualification retains a matching explicit CBOR
policy to preserve its authorized ICSS Git control. Frozen-release mode remains
disabled pending consumer qualification. [Adoption boundaries](../shared-tooling-adoption.md).
Snapshot/source verification, pins, both dependency boundaries, source checks,
release-adapter substitutes and local documentation links pass. Evidence is in
`target/evidence/shared-tooling-0307/`. Both lockfiles and pending runtime work
are preserved; no new runtime or release qualification is claimed.

IC Build 0.75.0 is adopted from crates.io for the native helper and selected
release-metadata CLI. Its removed shared-target inspection APIs had no Storage
callers. Only IC Build's version/checksum changed in this dependency update;
the independent lock and unrelated selections are preserved. The CLI is prepared
and admitted; 88 helper tests, strict Clippy, bootstrap-recovery substitutes,
both locked metadata observations, pin/production-graph checks and formatting
pass. Evidence: `target/evidence/ic-build-0750/`. This native tooling update
does not resolve the separate Platform export blocker or qualify publication.

The [type matrix](../audits/recurring/benchmark-full-catalogue-2026-11-05.md)
qualified 73 types / 565 cases across all five forms, with 46,604 measured calls
and final-state checks, 53 verified refusals and 18 non-Ord backend roles.
Selected results retain every case; no universal speedup is claimed. The earlier
reset baseline passed 3,196 calls, 824 iterator reruns and 480 Auth-shaped calls.
Canic configuration preparation and Auth's inner decoder remain outside scope.
The full type harness alone selects an eight-MiB stack; production settings are
unchanged. New raw comparison output uses `target/evidence/`.

Root qualification uses Wire 0.17.3, Runtime 0.19.2, Build/Host 0.74.4,
Metrics 0.29.3 and Logging 0.24.8. The independent runtime/comparison graph keeps
Wire 0.17.0, Runtime 0.19.0, Testkit 0.79.0 and PocketIC 16.1.0. Release-owned
version projection aligns the local Storage/Logging packages without silently
refreshing those supplier selections. No new macOS execution is claimed here.

## Historical evidence and preservation

The 2026-11-06 authorized cleanup retired 69 bulk/repeated experimental data and
prototype files and consolidated five superseded reports. The
[measurement index](../measurements/README.md) links current summaries and selected
results. All 565 cases retain backend status and selected Get/mutation costs;
complete per-run outputs are no longer tracked documentation. Historical figures
are not measurements of the current copied engines. Existing target run directories
were left untouched but remain removable by explicit cargo clean; no permanent
raw archive is promised and a digest cannot verify missing raw data.

Source, release/recovery records, small qualification tables and consumer inventory
remain intact. No cache, stash or target-tree cleanup was performed. Old collection
formats/prototypes are retired, rather than maintained as alternate implementations.
CI group qualification remains in `target/ci-group-selection/` and
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
