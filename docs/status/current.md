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

The pending 0.65.1 draft removes unused Native 0.79: Storage owns the memory
metering replacement and has no allocation-measurement callers. Original proof
is in `target/testkit-native-080-adoption/`. Package versions and the independent
Testkit CLI 0.79 selection are unchanged; cleanup remains uncommitted/unpublished.

The accepted audit fixes are implemented locally under the same 0.65.1 draft:
checked `MemoryRuntime::diagnostics()` reports current capacity and custody;
directory preparation drops the temporary indexed clone and carries its owned
image through metadata sizing; B-tree reuse splits oversized free blocks into
the requested block and reusable halves. Formats remain unchanged. Canic still
owns its diagnostic response migration; no sibling API or stored image was changed.
Original diagnostics/allocation evidence remains in
`target/storage-audit-improvements/`. The upstream follow-through adds optional
byte-budgeted map node caching, single-buffer routing views, unchanged-ancestor
write avoidance, `keys_range`, conservative live-record shrink reuse, fixed-slot
vector reads, reusable iterator buffers and cached extent translation. Generated
histories check 24,000 operations against an ordered model and cold live/free
range validation. Cache hits retain shared-owner poison checks. Formats remain
unchanged and ordinary map caching starts disabled.

`target/upstream-improvements/` retains 85 passing library tests, four public
runtime tests, strict native Clippy, earlier native/Wasm benchmark checks and
pins/production-graph checks. Final original-Wasm qualification passes IO,
growth/refusal, all five structures, upgrade/rollback, directory faults, custody
and Logging in `runtime-final-qualified/`. It uses root Runtime 0.18.1 and the
independent 0.18.0 client graph with Testkit CLI 0.79 / server 16.1.0. The prior
artifact-path failure remains in `runtime-final/`. No full release gate, commit
or publication was made; previously pending changes remain preserved.

The follow-up iterator fix checks range bounds before external value IO and
defers the following subtree until another step is requested. Two regressions
failed before repair; forward/reverse and mixed-direction coverage now passes.
`target/storage-scan-audit/` retains those failures, all 40 collection test results,
strict native/Wasm Clippy and rebuilt original-Wasm installed qualification.
Traversal still reports corruption when the requested range actually reaches it.

Both catalogs now select published Wire/derive 0.15.0. Public codec error identities
change, while codec bytes and budgets remain unchanged. Core Storage, production
Wasm and the independent qualifier each use one Wire 0.15. The subsequent
Metrics 0.27.0 adoption aligns default native schema support with Wire 0.15 and
removes the old Wire/derive 0.14 entries. This updates Metrics Storage's public
arithmetic/error identity; hosts must align direct Metrics dependencies. The
delivered 0.66.0 notes include this adoption. Source and
qualification evidence is retained in `target/metrics-027-adoption/`.
Its 30-test/doctest, graph and native/Wasm Clippy proof remains there. Earlier Wire
source/installed qualification remains in `target/wire-015-adoption/`; timing
matrices retain their original dependency selection and were not rerun.

Shared Tooling 0.27.27 (`651f5f7`) was exported from clean committed source and
verified across 144 payloads. Build/metadata CLI are 0.74.0, Host libraries 0.74.1,
Testkit CLI 0.79.0, and Platform/PocketIC 0.18 with server 16.1.0. Rust
1.99.1 remains the pinned compiler and support floor. Optional group validation
plans and historical-cache reclamation are not enabled. See the
[adoption boundary](../shared-tooling-adoption.md) for authoritative selections.

Earlier composition and installed proof remains in `target/issue-152/` and
`target/issue-152-runtime.bvHm54/`; tooling and B-tree proof remains under
`target/tooling-platform-adoption/` and `target/btree-runtime.WUAr9L/`.
New Linux qualification does not establish native macOS or optimizer-matrix results.

## Benchmark evidence and remaining costs

The independent benchmark covers 28 cases / 46 type-size combinations and all
five supported structures. It model-checks complete results and reports physical
stable pages as well as instructions. Published 0.7.2 uses Platform 0.17's facade;
the optional comparison-only Git control pins upstream `b5d1650`. Production has
no upstream dependency. See the
[methodology](../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md).

Final matched upstream-HEAD comparisons: `target/storage-comparison/attempt.ey0896/`
(Storage cache 128 KiB, upstream 32 slots) and `attempt.PkfT47/` (both disabled).
Each passes 4,212 measured calls and final model checks using identical Wasm hashes:
4,096 rows, integer maps plus six byte-value sizes through 4 KiB, one shuffled
population seed, three mutation rounds, uniform/hot-16/Zipf keys, batches 1/32.
The cache controls are not equal heap budgets. Storage's single-read cache gains
are 1.09× uniform, 30.02× hot-16 and 2.06× Zipf; batch-32 gains are 1.46×, 9.54×
and 2.07×. These measure collection intervals, not complete endpoint throughput.

With caches enabled, Storage/upstream ratios for uniform single operations are
1.8125 for integer Get, 1.1092 for Replace, 0.4806 for a one-entry bounded scan,
and 0.3987 for a 4 KiB-value Get. Upstream wins others decisively: the 4 KiB-value
missing-key trace costs Storage 176,082 instructions versus 2,502. Integer maps
reserve 6 versus 129 stable pages, but 4 KiB values reserve 519 versus 385.
Whole-canister reservation is not live payload. Cache placement, large-record
slack, directory snapshots and unreclaimed tails remain costs; no general
superiority, GiB-scale qualification or 2× whole-call guarantee is established.

Earlier draft proof remains in `attempt.NgZs3N/` and `attempt.VWaeMi/` (cached/
uncached Map and Vector, 828 calls each) and `attempt.i6I874/` (all 28 cases,
64 rows, uniform, published 0.7.2, 2,080 calls). These precede the final conservative
shrink adjustment. Historical expanded evidence remains in `attempt.97I6WQ/`
(4,160 calls) and `attempt.RfU4TW/` (256 rows, three seeds, 12,480 calls).
The original response-budget failure remains in `attempt.X5CMF4/`; old 14-case
4,096-row results in `attempt.NJabku/` do not qualify the expanded matrix.

The pre-delivery handoff and complete dirty-source backups are retained in
`target/storage-065-deploy/`. No build or historical evidence was removed.
