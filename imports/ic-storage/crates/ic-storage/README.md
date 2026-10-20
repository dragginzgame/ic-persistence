# IC Storage

Governed stable collections for Internet Computer applications. The supported
public surface is `ic_storage::adapter`: `StableBTreeMap`, `StableBTreeSet`,
`StableCell`, `BoundedCell`, `StableVec`, `StableMinHeap`, `StableLog`, their supported
iterators, `BTreeMapRegion`, `RegionGrowError`, `LogWriteError`, `Memory`, `Storable`,
`Bound` and `storable`. These wrappers preserve upstream
`ic-stable-structures` formats, initialization behavior and failure contracts.
Consumers use the adapter rather than depending directly on the substrate.

The library requires Rust 1.91 or later.

```rust
use ic_storage::adapter::{Bound, Memory, StableBTreeMap, Storable};

# fn example<M: Memory>(runtime: &ic_memory::MemoryRuntime<M>) -> Result<(), Box<dyn std::error::Error>> {
let _: Bound = <u64 as Storable>::BOUND;
// The application has already admitted and committed this allocation.
let memory = runtime.open_memory("app.rows.v1")?;
let mut map = StableBTreeMap::<u64, u64, M>::init(memory);
map.insert(1, 42);
assert_eq!(map.get(&1), Some(42));
drop(map);
let map = StableBTreeMap::<u64, u64, M>::load(runtime.open_memory("app.rows.v1")?);
assert_eq!(map.get(&1), Some(42));
# Ok(())
# }
```

The dependency direction is application → IC Storage → IC Memory. Collections
consume non-cloneable `ic_memory::RuntimeMemory<M>` allocation leases. The
remaining generic `M` identifies the runtime backing; it does not permit an
arbitrary collection-memory implementation. Allocation authority, IDs and runtime
bootstrap belong to IC Memory. A live lease prevents another open of the same
allocation through its runtime until the collection is dropped. A log consumes
two distinct leases. Hosts retain exclusive backing custody across runtimes;
lease tracking does not coordinate independent runtimes over aliased backing.

```compile_fail
use ic_memory::VectorMemory;
use ic_storage::adapter::StableBTreeMap;
let _ = StableBTreeMap::<u64, u64>::new(VectorMemory::default());
```

No raw backing constructors, memory manager or unrestricted upstream module is
exported. Wrappers expose neither `Deref`, raw-memory access nor extraction.
Drop a collection before reopening its allocation through IC Memory. Map
`reserve_pages` and vector `reset` consume and return the collection while
retaining its lease; reservation also returns a typed growth result. Map `clear`
removes entries in place. These operations expose no memory handle. `Memory` is
the exact upstream trait for generic backing bounds; implementing it grants no
allocation authority.

Map `reserve_pages` takes an absolute virtual extent in 65,536-byte Wasm pages.
It never shrinks memory and performs no growth or writes when already sufficient.
`allocated_pages` also consumes and returns the map to observe its virtual extent
without extracting the lease. Neither operation promises entry capacity or free
space. Callers own their byte/page budget and must qualify it against their codecs;
upstream node-layout constants are not a supported entry-reservation guarantee.

`BoundedCell` supports `init`, `get` and `set`, enforcing declared maximum or
exact encoded sizes before writes and before decoding retained values. It rejects
unbounded types. Rejected replacements preserve cached values and stored bytes;
invalid retained values refuse opening without rewriting bytes or leaking leases.
It preserves upstream cell bytes and initialization behavior, including initializing
empty memory or memory without the cell magic. Applications retain codec, schema
and recovery ownership; ordinary `StableCell` retains upstream's unchecked bounds.

## Retained trees inside a shared allocation

`BTreeMapRegion<K, V, M>` accepts an `Rc<RuntimeMemory<M>>` and an absolute,
half-open allocation-page range. It retains the original lease alongside the
application's disjoint frame owner. Construction performs no IO and grants no
new allocation authority. The view implements neither `Memory` nor raw extraction.
Every collection operation reloads the current root and allocator. `keys_range`
returns an owned, ordered snapshot, so application/checkpoint views can interleave
mutations without keeping stale roots or iterators. Applications choose and enforce
their namespace ranges; `len` counts the whole physical tree.

For `IcyDB`'s retained control layout, select `257..4_194_304`. Bytes have exactly
the upstream `RestrictedMemory` offset and B-tree format. Keep the existing
commit/boot/startup frame access through the same lease below page 257. No data
copy, allocation reassignment or migration is required for this upstream layout.
This interface does not support the retired independent tree format.

```rust
use ic_memory::RuntimeMemory;
use ic_storage::adapter::{BTreeMapRegion, Memory};
use std::rc::Rc;

fn inspect_control<M: Memory>(
    lease: Rc<RuntimeMemory<M>>,
) -> Result<Option<Vec<u8>>, &'static str> {
    let view = BTreeMapRegion::<[u8; 33], Vec<u8>, M>::new(lease, 257..4_194_304);
    if view.allocated_pages() == 0 {
        return Ok(None);
    }
    // Preserve the consumer's bounded checks before infallible tree loading.
    let mut frame = [0; 4];
    view.read(0, &mut frame);
    if frame != *b"BTR\x02" {
        return Err("invalid retained tree framing");
    }
    view.read(52, &mut frame);
    if frame != *b"BTA\x01" {
        return Err("invalid retained allocator framing");
    }
    // The consumer must still validate this header's schema/CRC, namespaces
    // and record limits before accepting the control store.
    Ok(view.get(&[0; 33]))
}
```

Use `initialize` only on a zero-page region; it refuses nonempty memory without
writing. Retained trees require admission followed by ordinary view operations.
The application keeps its existing semantic-header, namespace and capacity checks.
Malformed retained framing must return its corruption error before loading; the
view does not catch upstream panics or implement application recovery.

`read` checks the entire relative span against the allocated region before IO.
`reserve_pages` takes a relative minimum page extent and returns `RegionGrowError`
for region limits or typed IC Memory growth refusal. A rejected reservation
preserves bytes and allocation accounting; a satisfied request performs no IO.
Initialize empty regions before reserving. Reservation is a caller-qualified byte
budget, with no entry-capacity or transactional-insert guarantee. Collection IO
and growth stay within the selected region, including when adjacent frames exist.
Do not mutate tree bytes through the retained lease or reenter collection access
from record codecs. Drop all views and the caller's last `Rc` to release the lease.

```compile_fail
use ic_memory::VectorMemory;
use ic_storage::adapter::BTreeMapRegion;
use std::rc::Rc;
let _ = BTreeMapRegion::<u64, u64>::new(Rc::new(VectorMemory::default()), 0..8);
```

```compile_fail
fn extract(view: ic_storage::adapter::BTreeMapRegion<u64, u64>) {
    let _ = view.into_memory();
}
```

```compile_fail
fn raw(view: ic_storage::adapter::BTreeMapRegion<u64, u64>) {
    let _ = ic_storage::adapter::Memory::size(&view);
}
```

```compile_fail
use ic_storage::adapter::{MemoryManager, DefaultMemoryImpl};
```

```compile_fail
fn extract(map: ic_storage::adapter::StableBTreeMap<u64, u64>) {
    let _ = map.into_memory();
}
```

```compile_fail
fn bypass(map: &ic_storage::adapter::StableBTreeMap<u64, u64>) {
    let _: &ic_stable_structures::BTreeMap<u64, u64, ic_memory::RuntimeMemory<ic_memory::DefaultMemoryImpl>> = map;
}
```

```compile_fail
fn duplicate(memory: ic_memory::RuntimeMemory<ic_memory::DefaultMemoryImpl>) {
    let _ = memory.clone();
}
```

The former independent B-tree implementation has been removed. Its format is
incompatible with upstream collections; the adapter must not reopen those bytes.

Licensed under MIT; see LICENSE.
