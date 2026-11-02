# Storage release notes

Current releases use the Storage naming. [Earlier published history](https://github.com/dragginzgame/ic-storage/blob/23deb81c48d58558ac3b0ce36bf266ddc25dca91/releases/persistence/CHANGELOG.md)
retains its original package and release identities.

## [0.65.3] - Unreleased

### 🔧 Changed

- Scale optional map cache capacity with its byte budget and protect upper routing
  nodes from deeper-node collisions. Keep cache accounting and owner poison checks.
- Materialize mutable routing nodes only when they change; propagate child counts
  to avoid redundant rebalance probes. Public APIs and stored formats are unchanged.
  [Details](0.65.md).

### 🧪 Testing

- Correct comparison phase semantics: missing removals use separate keys, value
  growth precedes removal, and reinsertion follows removal. Check phase preconditions
  before execution and explicitly clean up inserted keys between rounds.
- Name above-population lookups `MissAbove`; retain historical mixed-trace results
  without reinterpreting their old labels. Add native cache and node-copy counters.

## [0.65.2] - 2026-11-02

### 🐛 Fixed

- Use Runtime >=0.18.1's safe IC0-only stable64 backing on Wasm. Default Storage
  no longer activates execution, CDK, guard, Types or Candid; native production
  remains Runtime-free. APIs and stored formats are unchanged. [Details](0.65.md).

## [0.65.1] - 2026-11-02

### 🔧 Changed

- Add checked, read-only allocation and physical-capacity diagnostics, including
  directory reservations and live lease counts. [Details](0.65.md).
- Simplify directory preparation by removing temporary indexed state and carrying
  the owned snapshot through reservation sizing.
- Split oversized free B-tree blocks when reusing them for smaller records,
  retaining the unused halves for later allocations. Stored formats are unchanged.
- Add opt-in byte-budgeted map node caching and bounded key-only map ranges.
  Read traversal retains one validated node buffer; mutations avoid unchanged
  ancestor writes and release excess capacity after substantial record shrinkage.
- Read fixed-size vector slots in one operation and reuse iteration buffers.
  Cache adjacent extent translations while preserving custody and bounds checks.

### 🧪 Testing

- Generate and minimize mutation/reopen histories with live/free range checks.
  Extend comparisons with a pinned upstream-head control, shuffled populations,
  Zipf access, repeated mutation rounds and explicit map cache controls.

### 🐛 Fixed

- Stop bounded scans before reading excluded external values, and defer reading
  following subtrees until another entry is requested. Unrelated corruption no
  longer fails an otherwise valid bounded scan.

### 🧹 Cleanup

- Remove the unused Testkit Native development dependency. Native backing IO
  metering already uses Storage's `test-support` helpers. [Details](0.65.md).

## [0.65.0] - 2026-11-02

### ⚠️ Breaking

- Adopt Wire 0.15 for public codec errors; hosts exchanging those types must
  align their Wire dependency. Codec bytes and budgets are unchanged.
- Replace the AVL engine with a multi-entry B-tree shared by Map, Set and regions.
  The new `ICMAP002` format requires explicit fresh/reset collection state and
  updated region framing admission. Public Rust APIs remain unchanged. [Details](0.65.md).

### 🔧 Changed

- Keep small values inline and load larger values separately; membership and
  key-only scans avoid unrelated payloads. Skip unchanged root metadata on replacement.
- Adopt Shared Tooling 0.27.27, Runtime/Platform 0.18, Build 0.74.0,
  Host libraries 0.74.1 and Testkit 0.79.0. Preserve the comparison-only
  Platform 0.17 stable-structures facade; production imports Runtime directly.
- Resolve the Platform 0.18 executor-guard conflict by using Runtime 0.18 directly
  ([#152](https://github.com/dragginzgame/ic-storage/issues/152)); qualify composition
  with the facade's runtime feature on native and Wasm.
- Restrict Storage's Runtime dependency to Wasm, removing it from native production.
  IC0-only Wasm backing remains blocked by [Platform #135](https://github.com/dragginzgame/ic-platform/issues/135).

### 🧪 Testing

- Add native `testing::MeteredMemory` and `MemoryIoCounts` under `test-support`,
  preserving backing custody, shared counters, reset and overflow detection.
- Expand installed comparisons to every supported structure and 28 concrete cases,
  including signed/wide keys, float vectors, inline-boundary and 4 KiB values,
  endpoint/bounded scans, growth/shrink and removal/reinsertion. Report physical
  stable pages alongside instructions.

## [0.64.0] - 2026-11-02

### ⚠️ Breaking

- Return typed codec/collection errors, including fallible map/set iteration and
  lazy value decoding. Require an explicit byte ceiling for cell loading.
- Protect cell/vector payloads with checksums (`ICCELL01` / `ICVECC01`). Reuse
  inactive directory arenas (`ICROOTR1` / `ICDIRR01`) and coalesce contiguous
  allocation growth. This extends the pending fresh/reset-backing hard cut.
- Extract the AVL deletion successor in one traversal. Native tests cover
  read-only error paths, repeated arena reuse and interrupted reuse/reopen.

- Rename the repository to `ic-storage` and release groups to `storage`,
  `metrics-storage` and `logging-storage`. Use the new adapter package names;
  published history and prior release receipts retain their original identities.

- Consolidate allocation and collections into `ic-storage`. Remove the `ic-memory`
  package; hosts and adapters must import runtime, key and lease APIs from
  `ic_storage`. No compatibility crate remains. [Details](0.64.md).

- Restrict collections to sibling-used Cell, Map, Set, Vector and map regions.
  Remove `StableLog`, `StableMinHeap`, `LogIter` and `LogWriteError`; no sibling
  application users were found in the local checkout scan.

- Replace the fixed-slot manager, upstream collections and codec/backing traits
  with owned named allocations and collection engines. Remove
  `ic-stable-structures`; adopt `StorageConfig`, namespace `AllocationGrants`
  and opaque u64 IDs. Fresh/reset whole backing is required: no legacy decoders
  or automatic reset. [Migration and limits](0.64.md).

### 🧹 Cleanup

- Avoid rewriting unchanged map ancestors on replacement, retain traversal
  cursors for scans, and index allocation extents for address translation.
  These changes preserve the current owned formats and public APIs.

- Share one owned AVL engine across Map, Set and regions; use bounded indexed
  slots for Vector. Retain exclusive custody, staged writes and corruption checks.

### 🧪 Testing

- Add `make benchmark-storage` for matched installed map operations against
  DFINITY, with validated results and retained instruction samples. The comparison
  stays outside production dependencies and release gates. [Methodology](../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md).
