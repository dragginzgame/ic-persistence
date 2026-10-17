//! Public collection adapters require committed IC Memory allocations.
use super::*;
use ic_memory::{
    MemoryManagerConfig, MemoryRuntime, RuntimeOpenError, VectorMemory, testing::fixture_runtime,
};

fn runtime(backing: VectorMemory) -> MemoryRuntime<VectorMemory> {
    fixture_runtime(
        backing,
        "test",
        "test.",
        &[
            "test.map.v1",
            "test.set.v1",
            "test.cell.v1",
            "test.vec.v1",
            "test.heap.v1",
            "test.log.index.v1",
            "test.log.data.v1",
        ],
        MemoryManagerConfig::default(),
    )
    .unwrap()
}

#[test]
fn collections_reopen_through_committed_runtime_allocations() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    assert!(matches!(
        runtime.open_memory("test.unknown.v1"),
        Err(RuntimeOpenError::StableKeyNotCommitted(_))
    ));
    let mut map = StableBTreeMap::<u64, u64>::new(runtime.open_memory("test.map.v1").unwrap());
    map.insert(7, 11);
    let mut set = StableBTreeSet::<u64>::new(runtime.open_memory("test.set.v1").unwrap());
    set.insert(9);
    let mut cell = StableCell::new(runtime.open_memory("test.cell.v1").unwrap(), 1u64);
    cell.set(3);
    let vector = StableVec::new(runtime.open_memory("test.vec.v1").unwrap());
    vector.push(&5u64);
    let mut heap = StableMinHeap::new(runtime.open_memory("test.heap.v1").unwrap());
    heap.push(&8u64);
    heap.push(&2);
    let log = StableLog::new(
        runtime.open_memory("test.log.index.v1").unwrap(),
        runtime.open_memory("test.log.data.v1").unwrap(),
    );
    log.append(&13u64).unwrap();
    for key in [
        "test.map.v1",
        "test.set.v1",
        "test.cell.v1",
        "test.vec.v1",
        "test.heap.v1",
        "test.log.index.v1",
        "test.log.data.v1",
    ] {
        assert!(
            matches!(
                runtime.open_memory(key),
                Err(RuntimeOpenError::AlreadyOpen { .. })
            ),
            "{key}"
        );
    }
    drop((map, set, cell, vector, heap, log));
    for key in [
        "test.map.v1",
        "test.set.v1",
        "test.cell.v1",
        "test.vec.v1",
        "test.heap.v1",
        "test.log.index.v1",
        "test.log.data.v1",
    ] {
        drop(runtime.open_memory(key).unwrap());
    }
    drop(runtime);

    let runtime = self::runtime(backing);
    let map = StableBTreeMap::<u64, u64>::load(runtime.open_memory("test.map.v1").unwrap());
    let mut iter: MapIter<'_, u64, u64> = map.iter();
    assert_eq!(iter.next().unwrap().into_pair(), (7, 11));
    let set = StableBTreeSet::<u64>::init(runtime.open_memory("test.set.v1").unwrap());
    let mut iter: SetIter<'_, u64> = set.iter();
    assert_eq!(iter.next(), Some(9));
    let cell = StableCell::init(runtime.open_memory("test.cell.v1").unwrap(), 99u64);
    assert_eq!(*cell.get(), 3);
    let vector = StableVec::<u64>::init(runtime.open_memory("test.vec.v1").unwrap());
    assert_eq!(vector.get(0), Some(5));
    let mut heap = StableMinHeap::<u64>::init(runtime.open_memory("test.heap.v1").unwrap());
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(8));
    let log = StableLog::<u64>::init(
        runtime.open_memory("test.log.index.v1").unwrap(),
        runtime.open_memory("test.log.data.v1").unwrap(),
    );
    let mut iter: LogIter<'_, u64> = log.iter();
    assert_eq!(iter.next(), Some(13));
}
