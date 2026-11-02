# Current handoff

## Delivered Storage 0.65.0

Storage **0.65.0** is published with Wire 0.15.0 and Runtime 0.18.0. Release commit
`84a7ecf6ed6517b989848c8e953d93fb5f561505` and annotated tag `storage-v0.65.0`
were atomically pushed through the standard group release workflow. The complete
local gate passed in 200 seconds, followed by prepared/final archive qualification.
Fresh sparse-index and downloaded registry archive checks agree with the receipt:
`c60314281e24c326e9dc938a29122763be56e6c3d76031be64221f38a6b5fb42`.
Version 0.65.0 is unyanked. Evidence is in `target/storage-065-deploy/` and
`target/release-validation/storage-0.65.0.json`; do not repeat release/upload.
Metrics Storage 0.66.0 is also delivered below; Logging Storage 0.64.0 remains pending.
No sibling release or migration is included. Preserve all earlier release plans,
archives, failure logs and caches.

The first release gate was interrupted before version preparation or push;
its log remains at `target/release-validation/attempts/verify.Pjm5qj`. The successful
replacement gate is `verify.XZLTRM`; no failed or interrupted evidence was removed.
The retired history directories remain available through the current ledgers'
revision-bound Git links; the maintainer explicitly approved their local deletion.

## Delivered Metrics Storage 0.66.0

Published `ic-metrics-storage` 0.66.0 selects Metrics 0.27, Wire 0.15 and Storage
0.65. Release commit `64777ae7a5ca916ecfcf7db94342b22008dc02c9` and annotated tag
`metrics-storage-v0.66.0` were atomically pushed. The complete local gate passed
in 216 seconds, followed by final archive qualification. Fresh index/download
checks match the qualified checksum, and the version is unyanked:
`8745c24fa8f8cd30635edf97d27644aae7c677e560826051e651e57f68729645`.
A registry-only consumer passes metric save/reopen and native Wire schema tests,
Wasm compilation and single-provider graph checks on native/Wasm. Evidence is in
`target/metrics-storage-066-deploy/` and the group final receipt under
`target/release-validation/`. Do not repeat release/upload. Core Storage and
Logging versions are unchanged; no sibling deployment or data migration ran.

## Contracts and completed source work

Map, Set and regions use the degree-six B-tree `ICMAP002`, replacing the released
AVL format. Hosts must explicitly rebuild/reset affected state and update region
framing admission. No legacy decoder or automatic reset exists. Public collection
signatures, allocation governance and Cell/Vector formats remain unchanged.
Small values up to 32 encoded bytes stay inline; external values are independently
checksummed. Membership and key-only scans do not audit external value integrity.
Mutations stage writes and admit capacity first; native backing failures poison
the owner, while IC traps retain host rollback. See the
[0.65 notes](../../releases/storage/0.65.md), [safety contract](../../SAFETY.md) and
[design provenance](../storage/upstreams.md).

Runtime 0.18 is selected directly on Wasm; native production excludes Platform,
Runtime, Types and the executor guard. This addresses
[Storage #152](https://github.com/dragginzgame/ic-storage/issues/152), now closed.
The isolated registry-only consumer of Storage 0.65.0 and Platform 0.18 passes
native governed-cell create/reopen and Wasm compilation with one Runtime/exact
guard 0.18.0 and Wire 0.15.0. It has no path overrides. The maintained source
composition fixture retains its own graph checks; registry Storage 0.64 still
has the old executor-guard conflict. Consumers must adopt the corrected release.

Wasm still enables Runtime's execution feature: the selected 0.18 API only exposes
stable-memory size publicly through IC0. [Platform #135](https://github.com/dragginzgame/ic-platform/issues/135)
has a separately prepared owner fix; adopt a qualified published release before
narrowing this dependency. [Relay #21](https://github.com/dragginzgame/ic-relay/issues/21)
remains downstream work. No local supplier patch or sibling edit is selected.

The reviewed native-metering contribution is applied under `test-support` as
`testing::{MeteredMemory, MemoryIoCounts}`. It preserves physical backing identity,
shared counters/reset, refused growth and sticky overflow. Allocation measurement
remains with Testkit Native. Its original patch and five passing metering/custody
tests, strict native/Wasm Clippy and formatting evidence are retained in
`target/metered-memory-adoption/`. Scheduling's separate migration was not applied.

## Tooling and qualification

Both catalogs now select published Wire/derive 0.15.0. Public codec error identities
change, while codec bytes and budgets remain unchanged. Core Storage, production
Wasm and the independent qualifier each use one Wire 0.15. The subsequent
Metrics 0.27.0 adoption aligns default native schema support with Wire 0.15 and
removes the old Wire/derive 0.14 entries. This updates Metrics Storage's public
arithmetic/error identity; hosts must align direct Metrics dependencies. The
delivered 0.66.0 notes include this adoption. Source and
qualification evidence is retained in `target/metrics-027-adoption/`.
All 30 Metrics Storage tests and three doctests pass, including native schema
composition through the adapter's Wire identity. Strict native/Wasm Clippy,
single-Metrics/single-Wire graph admission on both targets, production boundaries,
pins, formatting and documentation checks pass. The independent workspace lock
is unchanged; only Metrics and obsolete Wire records change in the root lock.
The preceding Wire adoption passed native product tests (112), eight
benchmark/composition tests and the selected native/Wasm graph/Clippy checks pass;
source comparison and registry evidence are in `target/wire-015-adoption/`.
The rebuilt original-Wasm installed IO, growth/refusal, reopen, upgrade, rollback,
custody, all five structures and Logging checks pass with Wire 0.15 in its
`runtime/` directory. Benchmark native and both Wasm backend Clippy pass; earlier
timing matrices retain their old dependency selection and were not rerun.

Shared Tooling 0.27.27 (`651f5f7`) was exported from clean committed source and
verified across 144 payloads. Build/metadata CLI are 0.74.0, Host libraries 0.74.1,
Testkit Native/CLI 0.79.0, and Platform/PocketIC 0.18 with server 16.1.0. Rust
1.99.1 remains the pinned compiler and support floor. Optional group validation
plans and historical-cache reclamation are not enabled. See the
[adoption boundary](../shared-tooling-adoption.md) for authoritative selections.

Focused native product tests, release-helper tests, production graph/refusal
checks and selected Clippy passed. Latest original-Wasm installed IO, zero/refused
growth, same-format reopen/upgrade, rollback, custody, all five structures across
1,031 allocations and Logging passed in `target/issue-152-runtime.bvHm54/`.
Composition and graph evidence is in `target/issue-152/`; earlier tooling and
B-tree proof remains under `target/tooling-platform-adoption/` and
`target/btree-runtime.WUAr9L/`. These earlier proofs do not assert a completed
release gate, optimized-Wasm matrix or native macOS qualification.

## Benchmark evidence and remaining costs

The independent benchmark covers 28 cases / 46 type-size combinations and all
five supported structures. It model-checks complete results and reports physical
stable pages as well as instructions. Only its DFINITY control retains Platform
0.17's stable-structures facade. See the
[methodology](../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md).

Expanded smoke: `target/storage-comparison/attempt.97I6WQ/`, 4,160 calls.
Expanded 256-row/three-seed run: `target/storage-comparison/attempt.RfU4TW/`,
12,480 calls and 2,080 summary rows, with source checks before and after execution.
The initial Wire response-budget failure is retained in `attempt.X5CMF4/`;
bounded allowances and largest-response regression tests corrected the harness.
No expanded 4,096-row run was performed. Earlier 14-case 4,096-row results remain
in `attempt.NJabku/` and do not qualify the expanded matrix.

No universal speedup is claimed. In the expanded matrix Storage/DFINITY ratios
include 1.2257 for signed single reads, 1.9490 for fixed-256-byte single reads,
4.4260 for batched last-entry lookup and 6.9840 for 4 KiB Cell growth.
Whole-canister stable pages include reservation, not just live payload. Directory
snapshot mutation and unreclaimed allocation tails remain known costs.

The pre-delivery handoff and complete dirty-source backups are retained in
`target/storage-065-deploy/`. No build or historical evidence was removed.
