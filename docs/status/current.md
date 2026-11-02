# Current handoff

## Selected delivery

The authorized core release is Storage **0.65.0**, from `main` at `a9e6fe6` plus
this session's pending changes. The standard `RELEASE_GROUP=storage` minor
release and publication workflow owns validation, version preparation, Git effects
and registry admission. This handoff records preparation, not successful delivery;
the annotated tag, retained final receipt and registry checksum establish that.
Metrics Storage 0.66.0 and Logging Storage 0.64.0 remain separate pending releases.
No sibling release or migration is included. Preserve all earlier release plans,
archives, failure logs and caches.

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
[Storage #152](https://github.com/dragginzgame/ic-storage/issues/152) in source.
The separate unpublished composition fixture verifies candidate Storage plus
published Platform 0.18 with one Runtime/guard on native and Wasm, and tests
native governed-cell create/reopen. Registry Storage 0.64 still has the old
executor-guard conflict; verify the delivered archive before closing the issue.

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
