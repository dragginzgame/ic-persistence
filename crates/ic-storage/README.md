# IC Storage

Owned stable allocation and collections for Internet Computer applications.
Storage owns the backing interface, codec trait, named allocation directory,
namespace admission, exclusive leases and collection implementations. It does
not depend on `ic-stable-structures`. Rust 1.99.1 or later is required.

## Supported collections

| Public type | Implementation | Known sibling use |
| --- | --- | --- |
| `StableCell` | Cached singleton with explicit create/load/reset | Canic, Blob |
| `StableBTreeMap` | Owned degree-six B-tree with reusable blocks | Canic, `IcyDB`, Blob, Relay, Metrics and Logging Storage |
| `StableBTreeSet` | The same ordered-tree engine with unit values | Blob deletions |
| `StableVec` | Fixed-stride indexed slots with bounded values | Canic templates and authentication |
| `BTreeMapRegion` | Operation-local view of the owned tree inside a leased page range | `IcyDB` control storage |

Map, Set and regions share one B-tree with up to eleven entries per node.
Values up to 32 encoded bytes stay inline; larger values occupy separate
checksummed blocks. Routing reads skip external values; lookups load matching
values and value-producing scans load candidate payloads.
Membership and key-only scans do not load external values. Iterators keep bounded
routing-node stacks and support forward/reverse traversal. Stable Log and Min-heap
remain outside the scope.

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

The root directory, map allocator metadata and vector root use bounded IC Wire.
B-tree routing nodes use checked binary fields; external values retain their
application encoding. Both have separate checksummed envelopes.
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

Maps stage affected B-tree nodes, values and allocator metadata before writing,
and publish changed root metadata last. Power-of-two blocks are reused after
removal or relocation. In-place replacements skip unchanged ancestors and root
metadata. `clear` reuses the allocation from its data start. Splits, sibling
borrowing, merging and root collapse keep the tree balanced. These algorithms
follow established B-tree practice; see [design provenance](../../docs/storage/upstreams.md).

The pending 0.65 map format is `ICMAP002`. It does not read 0.64's `ICMAP001`
AVL images. Hosts must explicitly rebuild/reset Map, Set and region state and
update their framing admission. There is no automatic reset or compatibility
reader. The directory, Cell and Vector formats and public Rust APIs are unchanged.

`reserve_pages` takes an absolute virtual extent in 65,536-byte pages, not entry
capacity. `allocated_pages` observes that extent while retaining the lease.
Satisfied reservations do not grow or write. These APIs grant no raw-memory
extraction.

## Map regions

`BTreeMapRegion::new(Rc<RuntimeMemory<M>>, start..end)` selects a disjoint page
range within one admitted allocation. Construction performs no IO. `initialize`
requires an empty region. Operations load the current owned root, allowing
views to interleave without stale caches. The current marker is `ICMAP002`;
old upstream `BTR`/`BTA` framing is unsupported. `IcyDB` must update its framing
admission together with fresh storage for this hard cut.

`keys_range_limited(bounds, limit)` bounds the collected key count before
returning an owned snapshot. Resume using an excluded last key. Regions bound
all IO and growth to their selected extent; callers own disjoint application
frames and must not reenter collection access from codecs. Drop all views and
the last shared lease owner before reopening that allocation.

See [allocation and host integration](ALLOCATION.md).

Licensed under MIT; see LICENSE.

## Native memory metering

Enable `test-support` and use `ic_storage::testing::{MeteredMemory, MemoryIoCounts}`
for native backing IO observations. The wrapper implements Storage's `Memory`
and delegates `MemoryBacking` identity to the underlying backing, preserving
exclusive runtime custody across raw, wrapped and cloned aliases. Counters are
shared by clones; `reset()` clears observations without clearing bytes. Reads
and writes count attempts, while growth counts completed calls and successful
requested pages. Counters saturate and set `overflowed`; reject overflowed results
when reporting exact measurements. These observations are native IO counts,
not IC instruction or cycle costs. Allocation measurement remains in
`ic-testkit-native`.
