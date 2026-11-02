# IC Storage

Owned stable allocation and collections for Internet Computer applications.
Storage owns the backing interface, codec trait, named allocation directory,
namespace admission, exclusive leases and collection implementations. It does
not depend on `ic-stable-structures`. Rust 1.99.1 or later is required.

## Supported collections

| Public type | Implementation | Known sibling use |
| --- | --- | --- |
| `StableCell` | Cached singleton with explicit create/load/reset | Canic, Blob |
| `StableBTreeMap` | Owned AVL ordered tree with reusable blocks | Canic, IcyDB, Blob, Relay, Metrics and Logging Storage |
| `StableBTreeSet` | The same ordered-tree engine with unit values | Blob deletions |
| `StableVec` | Fixed-stride indexed slots with bounded values | Canic templates and authentication |
| `BTreeMapRegion` | Operation-local view of the owned tree inside a leased page range | IcyDB control storage |

The map retains its public ordered-map name; its implementation is an AVL tree,
not a copy of the upstream B-tree. Stable Log and Min-heap are outside the scope.
Map and set iterators support forward/reverse traversal and ordered bounds.
Iterators retain bounded pointer stacks after the initial seek, with linear
traversal work across a scan and no retained ancestor payloads.

`ic_storage::adapter` exports these collections, `MapIter`, `SetIter`, `MapEntry`,
`RegionGrowError`, `CollectionError`, `CodecError`, and the owned `Memory`, `Storable`, `Bound` and `storable`
interfaces. Backing implementations and runtime types live at the crate root.
Collections require non-cloneable `RuntimeMemory` leases; implementing `Memory`
or `Storable` does not grant access to an allocation.

```rust
use ic_storage::{AllocationGrants, GenericAllocationPolicy, MemoryAuthority,
    MemoryRequest, MemoryRuntime, SealedDeclarationSnapshot, StableKey, VectorMemory};
use ic_storage::adapter::StableBTreeMap;

let mut host = MemoryRuntime::new(VectorMemory::default())?;
let requests = SealedDeclarationSnapshot::new(vec![
    MemoryRequest::new("app", "app.rows.v1")?,
])?;
let grants = AllocationGrants::new(vec![MemoryAuthority::new("app", "app.")?])?;
host.bootstrap(&requests, &grants, &GenericAllocationPolicy)?;
let key = StableKey::parse("app.rows.v1")?;
let mut rows = StableBTreeMap::<u64, u64>::create(host.open_memory(&key)?)?;
rows.insert(7, 11)?;
drop(rows);
let rows = StableBTreeMap::<u64, u64>::load(host.open_memory(&key)?)?;
assert_eq!(rows.get(&7)?, Some(11));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Current storage contract

This is a hard cut. Previous manager, ledger, map, set, vector and region images
are retired. There is no legacy decoder or automatic reset. Hosts must install
fresh backing or explicitly retire the complete old storage image; never reset
only allocation governance while retaining unmanaged application data. Rebuild
consumers against Storage's owned codec and backing traits. Primitive numeric
codecs are little-endian; application codecs remain application-owned.

The root directory, map metadata/nodes and vector root use bounded IC Wire.
Checksums detect accidental corruption, not hostile changes by linked code.
Map node records have a 16 MiB encoded limit. Vector records require a finite
`Storable::BOUND`; each slot reserves its maximum width. Cell supports unbounded
values within its u32 envelope length and enforces finite bounds where declared.
Cells and vector slots also checksum their payloads. `StableCell::load(memory,
maximum_bytes)` requires a caller-selected ceiling before payload allocation.
The cell marker is `ICCELL01` with a 24-byte envelope; the vector marker is
`ICVECC01` with 12 bytes of length/checksum overhead per slot. These formats
require fresh/reset storage; earlier cell/vector bytes are not reinterpreted.

`Storable` encoding, decoding and bound checks return `Result<_, CodecError>`.
Collection creation, loading, lookup and mutation return `CollectionError`;
map/set iterators yield `Result` entries and fuse after an iteration error.
`MapEntry` value decoding is separately fallible. Handle errors explicitly;
Metrics/Logging Storage propagate them through `StoreError::Storage`.
Backing panics and buggy caller codecs remain host failures. Cached length and
value observations are not recovery APIs for a poisoned owner.

`create` requires empty allocation memory. `load` only reads and never repairs or
initializes a replacement. `reset` explicitly replaces contents in an admitted
allocation. Maps also support `clear`. All collection growth reserves
capacity before writes; rejected encoding/bounds or ordinary growth refusal
leave prior values intact. Native backing writes are not transactions. A backing
failure poisons the shared allocation owner; drop it and its leases before
reopening. Installed Wasm traps follow the host's rollback semantics.

Maps stage the affected AVL nodes and allocator metadata before writing, publish
the root last, and reuse power-of-two blocks on removal or replacement. `clear`
reuses the allocation from its data start. They do not load the complete map
into heap. This implementation has correctness qualification, not a production
performance-equivalence claim. Node reads still include encoded values; hosts
should measure their large-record and scan workloads. Existing-key replacements
skip unchanged ancestors and do not rebalance the tree. Two-child deletion
extracts and rebalances the successor path once, without searching for it again.

`reserve_pages` takes an absolute virtual extent in 65,536-byte pages, not entry
capacity. `allocated_pages` observes that extent while retaining the lease.
Satisfied reservations do not grow or write. These APIs grant no raw-memory
extraction.

## Map regions

`BTreeMapRegion::new(Rc<RuntimeMemory<M>>, start..end)` selects a disjoint page
range within one admitted allocation. Construction performs no IO. `initialize`
requires an empty region. Operations load the current owned root, allowing
views to interleave without stale caches. The current marker is `ICMAP001`;
old upstream `BTR`/`BTA` framing is unsupported. IcyDB must update its framing
admission together with fresh storage for this hard cut.

`keys_range_limited(bounds, limit)` bounds the collected key count before
returning an owned snapshot. Resume using an excluded last key. Regions bound
all IO and growth to their selected extent; callers own disjoint application
frames and must not reenter collection access from codecs. Drop all views and
the last shared lease owner before reopening that allocation.

See [allocation and host integration](ALLOCATION.md).

Licensed under MIT; see LICENSE.
