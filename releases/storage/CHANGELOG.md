# Storage release notes

Current releases use the Storage naming. [Earlier published history](../persistence/CHANGELOG.md)
retains its original package and release identities.

## [0.64.0] - Unreleased

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
