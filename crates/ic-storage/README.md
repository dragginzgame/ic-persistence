# IC Storage

Governed stable collections for Internet Computer applications. The supported
public surface is `ic_storage::adapter`: `StableBTreeMap`, `StableBTreeSet`,
`StableCell`, `StableVec`, `StableMinHeap`, `StableLog`, their supported iterators,
`LogWriteError`, `Storable` and `storable`. These wrappers preserve upstream
`ic-stable-structures` formats, initialization behavior and failure contracts.
Consumers use the adapter rather than depending directly on the substrate.

The library requires Rust 1.91 or later.

```rust
use ic_storage::adapter::StableBTreeMap;

# fn example(runtime: &ic_memory::MemoryRuntime<ic_memory::DefaultMemoryImpl>) -> Result<(), Box<dyn std::error::Error>> {
// The application has already admitted and committed this allocation.
let memory = runtime.open_memory("app.rows.v1")?;
let mut map = StableBTreeMap::<u64, u64>::init(memory);
map.insert(1, 42);
assert_eq!(map.get(&1), Some(42));
drop(map);
let map = StableBTreeMap::<u64, u64>::load(runtime.open_memory("app.rows.v1")?);
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
Drop a collection before reopening its allocation through IC Memory.

```compile_fail
use ic_storage::adapter::{Memory, MemoryManager, DefaultMemoryImpl};
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

```compile_fail
use ic_storage::{BTreeMap, Comparator, Config, Entry, Error, Lexicographic, Range};
```

```compile_fail
use ic_storage::adapter::{BTreeMap, Comparator, Config, Entry, Error, Lexicographic, Range};
```

Licensed under MIT; see LICENSE.
