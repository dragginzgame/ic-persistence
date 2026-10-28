# Current handoff

Released source is 0.53.0 at `798b435adb0fcceea643a3d636c94315941d866a`.
[Its CI](https://github.com/dragginzgame/ic-persistence/actions/runs/40692276364)
passed all 10 supplier jobs. IcyDB reports successful published-registry adoption
for all nine grouping cases, its 10,001-row collection/upgrade case and 16 host
memory-admission cases, preserving original budgets. See
[#137 — Fail closed when multiple memory runtimes alias one stable backing](https://github.com/dragginzgame/ic-persistence/issues/137).

## Pending 0.53.1 teardown fix

The issue remains open for a new native consumer failure: Blob's released source
`47df5334d0c625dbf6081fd8e172f96c72764a7d` fails exact archive tests on both macOS
architectures in [run 40701957045](https://github.com/dragginzgame/ic-blob-storage/actions/runs/40701957045).
Fresh independent VectorMemory fixtures report `BackingAlreadyClaimed`.

The stale-claim window is now reproduced deterministically on Linux. A fixture
frees the last physical Rc identity allocation under the registry lock, preventing
a legitimate new claimant from masking the observation. The old implementation
still contains the freed identity after normal teardown and constructor panic.
This proves the lifetime defect without depending on allocator scheduling; the
exact macOS failures still require their native rerun for attribution/acceptance.

The local fix declares the shared claim before its backing, retiring it first
when the last runtime/handle drops. Construction owns the Rc backing before
acquiring its local claim, so cleanup drops the claim before freeing identity.
Fallible authority capture completes before the claim transfers to shared state.
Tests cover final runtime/handle release, typed growth refusal, initialization
and capture panics, and 8000 independent constructions across four native threads.
Duplicate-owner refusal and detached-handle custody remain enforced.

See [pending notes](../changelog/0.53.md#0531---unreleased). Package versions remain
0.53.0; both catalogs/locks, toolchains and real index are unchanged. Edits are
uncommitted; no release, publication, push or workflow dispatch is performed.

## Qualification and native acceptance

Evidence is retained at `/tmp/ic-persistence-teardown-137/`, including original
red tests, corrected runs, incoming locks, isolated Blob source and archive output.
72 Memory runtime cases, 16 fixture cases and 17 Storage adapter cases pass;
all nine custody cases also pass on actual Rust 1.91. Strict native Memory/test
and actual Wasm library Clippy pass. Rust 1.91 builds the Memory canister fixture;
installed PocketIC 16.1.0 qualification passes checkpoint faults, poisoned-handle
refusal, composed handles, final release and upgrade reopening.

The isolated Blob copy is the exact failing source, with an investigation-only
Cargo config patch selecting local candidate Memory 0.53.0. Canonical coordinated
archive generation, graph/lock admission, all targets, doctests and final source
integrity checks pass on Linux. Metadata proves the extracted tests resolve the
candidate Memory source. All 339 core cases pass, including the reported missing
grant case; the normal parallel runner is retained. Blob source is not edited,
its original sibling checkout stays untouched, and the override is not publication.
Archive evidence is under `blob-source/target/package/blob-verification.Pru0iM/`
beneath the investigation directory. The patch remains open for delivery and
required supplier/native checks, including Blob archive acceptance on both Macs.
No Linux or prior green run is relabelled as new-patch macOS acceptance.

## Preserved runtime contract, tooling and recovery

The maintainer-selected 0.53.0 contract remains: construction, initial bootstrap,
new leases, diagnostics and growth perform full metadata validation. Size and
safe/raw IO trust exclusive backing custody between checkpoints, check virtual
bounds and refuse poisoned owners. Raw aliases/unmanaged managers are prohibited,
including over excluded IDs; truthful identities and one Memory crate identity
remain required. No hot-path scan, new unchecked mode or stored format is added.

Original cost/consumer measurements and their source/lock/Wasm hashes remain at
`/tmp/ic-persistence-cost-137/`. Earlier supplier and fixture evidence remains at
`/tmp/ic-persistence-issue-137/` and `/tmp/ic-persistence-fixture-custody-fix/`.
Platform stays 0.7.3, Tooling/Build/Testkit 0.69.6, stable-structures 0.7.2 and
PocketIC client/server 16.1.0. Shared 0.27.3 remains reviewed at
`7ee5c83744aa822ecd070703dbf0a7d95076d5f6`. Adoption evidence remains at
`/tmp/ic-persistence-adoption-073-0696/`, `/tmp/ic-persistence-adoption-20261028/`
and `/tmp/ic-persistence-deps-20261028/`, with each original source identity.

Preserve all release journals, receipts, failed attempts and retained archives.
Original Memory/Storage history survives through the
[migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). Keep verified backups at
`/home/adam/projects/ic-persistence-retirement-2026-10-20/` and all mirrors,
exports and recovery plans. Original checkout retirement was authorized; last
GitHub deletions were refused for missing `delete_repo`, with no retry in scope.
