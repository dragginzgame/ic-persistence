# Storage release notes

Current releases use the Storage naming. [Earlier published history](https://github.com/dragginzgame/ic-storage/blob/23deb81c48d58558ac3b0ce36bf266ddc25dca91/releases/persistence/CHANGELOG.md)
retains its original package and release identities.

## [0.67.0] - 2026-11-04

### ⚠️ Breaking

- Adopt IC Wire 0.17 for StorageConfig::wire and Storage codec errors. Hosts exchanging Wire limits or
  errors must align their dependency to the new type identity. Codec bytes,
  budgets and stored layouts are unchanged; no backing reset is needed.
  Codec-only builds no longer select Wire's metadata derive. [Details](0.67.md).

## [0.66.3] - 2026-11-04

### ⚡ Performance

- Use a two-way Map/Set node cache so a protected ancestor can coexist with a
  colliding descendant, retaining the heap-byte budget and existing address hash.
  Selected baseline batch traces improve by up to 26.9%, with up to 1.2% warm
  and 1.5% cold whole-trace regressions. No allocation or persisted-format change.
  [Scope and tradeoffs](0.66.md).

- Replace inline Map values directly in encoded node frames, avoiding temporary
  buffers for unchanged entries. Preserve stored formats and integrity checks.
  Matched 256-row tests reduce inline replacement instructions by 3.7–7.5% for single
  updates and 19.5–26.6% for batches of 32; see [tradeoffs and scope](0.66.md).
- Decode Map/Set cursor keys directly from validated node buffers, avoiding a
  temporary encoded-key allocation. Key-only scans also skip unused inline-value
  buffers. Preserve range ordering and integrity checks. A matched 64-record run
  reduced scalar Set scan instructions by 7.9% and array-Map key scans by 15%.
  [Investigation](../../docs/audits/recurring/storage-icss-hotspots-2026-11-04.md).

### 🔧 Tooling

- Apply qualified group selection to private adapter PRs. Scope lint, feature,
  rustdoc and Wasm compilation to the admitted adapter in CI and release plans.
  Preserve full checks for core/shared/unknown changes and non-docs pushes, plus
  workspace archives, runtime qualification and native evidence obligations.
  [Selection and validation](../../docs/storage-release-validation.md).

### 🧪 Testing

- Add a focused small-record capacity/reuse study covering external-value
  allocation boundaries and metadata IO. Experimental denser layouts remain
  research only; production allocation policy and retained formats are unchanged.
- Expand ICSS comparisons with a size-boundary workload, sequential access and
  bounded byte vectors in the default matrix. Preserve matched operations and
  independent result validation. Add per-round summaries so aggregate medians
  do not hide first-round versus mutated-state costs; production behavior is unchanged.
  [Details](0.66.md).

## [0.66.2] - 2026-11-04

### 🔧 Changed

- Decode borrowed map-entry values without an intermediate encoded-buffer clone;
  preserve consuming decode, codec errors, public APIs and stored formats.
- Extend B-tree histories and the shared-backing study across changing key/value
  sizes. Bulk benchmarks now exercise actual growth and check IC reply limits.
  See the [ICSS re-audit](../../docs/audits/recurring/storage-icss-2026-11-03.md).

- Adopt Shared Tooling 0.29.4 (`628ccc7`), including the earlier UTF-8 evidence
  fixes and new test-layout, Metrics ownership and group-changelog rules.
- Guard Git-side release state against excessive retention and nested build caches,
  without deleting recovery evidence. Keep dependency-adoption evidence under `target/`.
- Move inline unit tests and private fixture helpers into `tests.rs` or `tests/`
  across Storage, its native helper and the independent qualification workspace.
  Preserve behavior and ordinary module discovery. CI-selection fixtures now
  remain under `target/ci-selection/`.

- Move remaining consumer fixture scratch under workspace `target/`, preserving
  failed evidence and explicit alias probes. Refresh the active audit guide for
  Storage's directory/lease model and Rust 1.99.1.

- Select eight-page buckets for the ICSS research comparison by default after
  matched 1/8/128-page measurements; retain explicit stock-128 and low-memory-1
  controls. Storage's production allocation policy is unchanged.

- Add a native shared-backing experiment for 32/64 allocations, mixed record
  sizes, interleaved growth and repeated churn, plus controlled external-value
  allocation-boundary probes. Keep production geometry and defaults unchanged.

## [0.66.1] - 2026-11-03

### 🔧 Changed

- Load the release stage catalog from admitted source at runtime, preserving
  standalone bootstrap and archive builds without embedding repository-root files.

- Qualify the native Rust group-proof collector and retained-proof recovery
  against real Shared runner/verifier evidence, including failures and tampering.
  [#156](https://github.com/dragginzgame/ic-storage/issues/156).

- Adopt Shared Tooling 0.28.2 (`5cf67f8`), centralize group evidence and
  baseline observation, and derive full dispatch from one stage catalog.
  Preserve release projection recovery, existing proof and pending work.

- Add consumer-informed `control`, `records` and `bulk` benchmark recipes,
  including byte vectors and opt-in 64 KiB/1 MiB records. Record sibling memory
  declarations and sizing boundaries; production defaults remain unchanged.

- Select release validation by proven change scope. Private adapter-only changes
  can omit unaffected checks; core/shared changes and repository CI retain the
  full roster. Group receipts preserve stage evidence for safe recovery.
  [Details](0.66.md).

## [0.66.0] - 2026-11-03

### ⚠️ Breaking

- Adopt Wire 0.16 for public codec errors and limits. Hosts exchanging those Rust
  values must align their Wire dependency; stored bytes and reopening stay unchanged.

### 🔧 Changed

- Adopt Shared Tooling 0.27.29: preserve implementation in authorized durable
  checkouts, require explicit direct-push and alternate-delivery authority, and
  reject incomplete tool-store inventories. Release planning supports private
  path-only development members; the tooling update preserves Storage's package APIs and formats.
  [Details](0.66.md).

## [0.65.5] - 2026-11-03

### 🧪 Testing

- Add an explicit ICSS allocation-bucket setting to the installed comparison
  across all supported structures; record it beside instruction and memory results.

### 🔧 Changed

- Adopt Platform Runtime 0.19 for IC0-only Wasm backing and Platform 0.19's
  fallible native qualification client. Preserve the optional execution boundary,
  public Storage APIs and stored formats; requalify installed lifecycle behavior.
- Encode B-tree nodes directly into their final checksummed buffer, removing a
  temporary payload allocation and copy. Membership checks skip value copies,
  and map reads decode inline values from the checked node buffer. Public APIs,
  stored formats and staged publication are unchanged. [Details](0.65.md).

## [0.65.4] - 2026-11-02

### 🔧 Changed

- Prepare and publish directory mutations under one exclusive store borrow,
  removing the copied base image and intermediate candidate indexes. Preserve
  backing verification, resource limits, poisoning and root-last publication.
- Admit source-projected release lockfiles with locked, offline Cargo fetching
  instead of a second dependency-update step. Keep exact source and archive checks.
  Public APIs and stored formats are unchanged. [Details](0.65.md).
- Reuse modified nodes and inspected siblings through B-tree deletion repair,
  and read external values into one checked payload buffer. Preserve staged
  publication, checksums and the current collection formats.

## [0.65.3] - 2026-11-02

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
