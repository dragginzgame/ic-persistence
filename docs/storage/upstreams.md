# Upstream ownership

Storage owns the allocator, backing trait, codec trait and Cell, Map, Set, Vector
and map-region implementations. No imported `ic-stable-structures` source,
re-export, feature or selected dependency remains. Production graph admission
forbids that package. The replacement is a complete retained-format hard cut.

IC Platform supplies Wasm stable-memory system access. IC Wire supplies bounded
directory and collection-record encoding. Storage owns their framing, checksums,
resource limits and allocation/collection invariants. Hosts own application
codecs, comparators, schema compatibility, lifecycle and recovery decisions.
See [Storage](../../crates/ic-storage/README.md) and
[allocation](../../crates/ic-storage/ALLOCATION.md).

Collections accept only admitted `RuntimeMemory` leases. `BTreeMapRegion` uses an
`Rc<RuntimeMemory>` over a caller-selected page range and reloads the owned tree
root per operation. Hosts must admit disjoint framing and update old `BTR`/`BTA`
checks to the current `ICMAP002` contract. No region allocates another identity.

Shared Tooling owns the reviewed engineering baseline, formatting, setup and
release runner. The snapshot remains unchanged by this rewrite. Build and Host
own reusable native metadata, filesystem, process and archive operations;
Storage's repo-tool retains consumer release policy and qualification receipts.
Testkit owns server selection/lifecycle; the independent runner owns installed
Storage and adapter scenarios. None of those native tools enters the production
canister graph. See [adoption boundaries](../shared-tooling-adoption.md).

The test-support `fixture_runtime` performs the same namespace admission and
bootstrap as production. Tests choose their own backing, stable keys and
`StorageConfig`. Sibling repositories remain separately responsible for adopting
the changed public types and explicit reset requirement.

## B-tree design provenance

The pending 0.65 engine applies the multi-entry B-tree design used by DFINITY's
`ic-stable-structures` 0.7.2: minimum degree six, binary search within nodes,
split/borrow/merge balancing and avoiding unrelated value reads. Reference
implementation: `src/btreemap.rs`, `src/btreemap/node.rs` and
`src/btreemap/node/v2.rs` in that published crate (DFINITY Foundation, Apache-2.0).
No upstream source files are vendored and no production dependency is added.
This is an independently implemented format, not a port of upstream bytes.

Storage uses bottom-up splits and underflow repair with staged mutations, a
checked compact routing encoding, inline values up to 32 bytes, and separately
checksummed larger values. It retains its existing governed memory and reusable
block allocator. DFINITY's exact node layout, overflow pages and bucket manager
are not reproduced. The comparison harness measures these different stacks;
results do not isolate the B-tree algorithm from allocation or integrity costs.

`ICMAP002` requires explicit replacement of earlier map/set/region state.
There is no legacy reader, implicit repair or automatic reset. Root loading
validates framing and bounds without scanning all nodes. Key-only operations
check routing records but deliberately do not audit external value payloads.

## Follow-through from upstream comparison

The 0.65.1 draft additionally studies upstream commit
[`b5d1650`](https://github.com/dfinity/stable-structures/commit/b5d1650463f477affb3d358e86b1113953fc9937),
including its configurable node cache, mutation invalidation, property tests and
cache-sensitive benchmarks. Published 0.7.2 remains a separate comparison control.

Local changes apply those mechanisms within Storage's current contracts:

- Validated routing buffers can be cached under an opt-in byte budget. Accounting
  uses retained vector capacities and cache bookkeeping rather than upstream's
  slot count and approximate heap estimate. Allocator overhead and iterator-held
  nodes remain outside that budget; this is not a whole-canister heap limit.
- One checksummed buffer supplies read-only entry views. Storage continues to
  check the complete routing record, while upstream can defer individual keys.
- Unchanged ancestors stay unwritten. Substantially shrinking records reuse an
  available smaller block and free the original whole block through staged free
  lists. Otherwise capacity stays reserved, preserving reuse under size churn.
  This differs from upstream's page-based unused overflow reclamation.
- Fixed-size vector IO and reusable iterator buffers retain Storage's per-slot
  length/index checksum; upstream's lighter encoding has different integrity costs.
- Translation caching retains allocation admission, range validation and global
  poisoning. It does not adopt upstream's fixed memory-ID/bucket manager.

These are independently implemented adaptations, not imported source or a claim
of general superiority. Bounded key-only ranges and generated model histories
close existing gaps. The matched installed comparison must establish performance
for each workload; removing instructions or reads locally does not prove a 2×
whole-call improvement. Production has no upstream dependency or format migration.
