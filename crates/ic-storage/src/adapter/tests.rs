//! Public collection adapters require committed IC Memory allocations.
use super::*;
use ic_memory::{
    MemoryManagerConfig, MemoryRuntime, RuntimeOpenError, VectorMemory, testing::fixture_runtime,
};
use std::{cell::Cell, rc::Rc};

mod cells;
mod regions;

const KEYS: [&str; 7] = [
    "test.map.v1",
    "test.set.v1",
    "test.cell.v1",
    "test.vec.v1",
    "test.heap.v1",
    "test.log.index.v1",
    "test.log.data.v1",
];

#[derive(Clone)]
struct ControlledMemory {
    backing: VectorMemory,
    refuse_growth: Rc<Cell<bool>>,
    growth_calls: Rc<Cell<u64>>,
}
impl ic_memory::MemoryBacking for ControlledMemory {
    fn backing_identity(&self) -> ic_memory::BackingIdentity {
        self.backing.backing_identity()
    }
}

impl Memory for ControlledMemory {
    fn size(&self) -> u64 {
        self.backing.size()
    }

    fn grow(&self, pages: u64) -> i64 {
        self.growth_calls.set(self.growth_calls.get() + 1);
        if self.refuse_growth.get() {
            -1
        } else {
            self.backing.grow(pages)
        }
    }

    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.backing.read(offset, dst);
    }

    fn write(&self, offset: u64, src: &[u8]) {
        self.backing.write(offset, src);
    }
}

fn runtime<M: ic_memory::MemoryBacking>(backing: M) -> MemoryRuntime<M> {
    fixture_runtime(
        backing,
        "test",
        "test.",
        &KEYS,
        MemoryManagerConfig::default(),
    )
    .unwrap()
}

#[test]
fn collections_reopen_through_committed_runtime_allocations() {
    let keys = KEYS.map(|key| ic_memory::StableKey::parse(key).unwrap());
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let mut map = StableBTreeMap::<u64, u64>::create(runtime.open_memory(&keys[0]).unwrap());
    map.insert(7, 11);
    let mut set = StableBTreeSet::<u64>::create(runtime.open_memory(&keys[1]).unwrap());
    set.insert(9);
    let mut cell = StableCell::create(runtime.open_memory(&keys[2]).unwrap(), 1u64);
    cell.set(3);
    let vector = StableVec::create(runtime.open_memory(&keys[3]).unwrap());
    vector.push(&5u64);
    let mut heap = StableMinHeap::create(runtime.open_memory(&keys[4]).unwrap());
    heap.push(&8u64);
    heap.push(&2);
    let log = StableLog::create(
        runtime.open_memory(&keys[5]).unwrap(),
        runtime.open_memory(&keys[6]).unwrap(),
    );
    log.append(&13u64).unwrap();
    for key in &keys {
        assert!(
            matches!(
                runtime.open_memory(key),
                Err(RuntimeOpenError::AlreadyOpen { .. })
            ),
            "{key}"
        );
    }
    drop((map, set, cell, vector, heap, log));
    for key in &keys {
        drop(runtime.open_memory(key).unwrap());
    }
    drop(runtime);

    let runtime = self::runtime(backing);
    let map = StableBTreeMap::<u64, u64>::load(runtime.open_memory(&keys[0]).unwrap());
    let mut iter: MapIter<'_, u64, u64> = map.iter();
    assert_eq!(iter.next().unwrap().into_pair(), (7, 11));
    let set = StableBTreeSet::<u64>::load(runtime.open_memory(&keys[1]).unwrap());
    let mut iter: SetIter<'_, u64> = set.iter();
    assert_eq!(iter.next(), Some(9));
    let cell = StableCell::<u64>::load(runtime.open_memory(&keys[2]).unwrap()).unwrap();
    assert_eq!(*cell.get(), 3);
    let vector = StableVec::<u64>::load(runtime.open_memory(&keys[3]).unwrap());
    assert_eq!(vector.get(0), Some(5));
    let mut heap = StableMinHeap::<u64>::load(runtime.open_memory(&keys[4]).unwrap());
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(8));
    let log = StableLog::<u64>::load(
        runtime.open_memory(&keys[5]).unwrap(),
        runtime.open_memory(&keys[6]).unwrap(),
    );
    let mut iter: LogIter<'_, u64> = log.iter();
    assert_eq!(iter.next(), Some(13));
}

#[test]
fn reservation_clear_and_reset_retain_exclusive_leases() {
    let runtime = runtime(VectorMemory::default());
    let mut map = StableBTreeMap::<u64, u64>::create(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap())
            .unwrap(),
    );
    map.insert(7, 11);
    let (map, result) = map.reserve_pages(8);
    result.unwrap();
    let before = runtime.memory_allocations().unwrap();
    let (mut map, result) = map.reserve_pages(u64::MAX);
    assert!(matches!(
        result,
        Err(ic_memory::RuntimeGrowError::BucketExhausted { .. })
    ));
    assert_eq!(map.get(&7), Some(11));
    assert_eq!(runtime.memory_allocations().unwrap(), before);
    assert!(matches!(
        runtime.open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap()),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    map.clear();
    assert!(map.is_empty());
    drop(map);
    let memory = runtime
        .open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap())
        .unwrap();
    assert!(memory.size() >= 8);
    let map = StableBTreeMap::<u64, u64>::load(memory);
    assert!(map.is_empty());
    let vector = StableVec::create(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.vec.v1").unwrap())
            .unwrap(),
    );
    vector.push(&5u64);
    drop(vector);
    let vector = StableVec::<u64>::reset(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.vec.v1").unwrap())
            .unwrap(),
    );
    assert!(vector.is_empty());
    assert!(matches!(
        runtime.open_memory(&ic_memory::StableKey::parse("test.vec.v1").unwrap()),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    drop(vector);
    let vector = StableVec::<u64>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.vec.v1").unwrap())
            .unwrap(),
    );
    assert!(vector.is_empty());
}

#[test]
fn reservation_is_observable_effect_free_when_satisfied_and_retryable_after_refusal() {
    let memory = ControlledMemory {
        backing: VectorMemory::default(),
        refuse_growth: Rc::new(Cell::new(false)),
        growth_calls: Rc::new(Cell::new(0)),
    };
    let runtime = runtime(memory.clone());
    let mut map = StableBTreeMap::<u64, u64, _>::create(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap())
            .unwrap(),
    );
    map.insert(7, 11);
    let (map, result) = map.reserve_pages(8);
    result.unwrap();
    let before = memory.backing.borrow().clone();
    let allocations = runtime.memory_allocations().unwrap();
    let growth_calls = memory.growth_calls.get();
    let (map, pages) = map.allocated_pages();
    assert_eq!(pages, 8);
    let (map, result) = map.reserve_pages(pages);
    result.unwrap();
    let (map, result) = map.reserve_pages(0);
    result.unwrap();
    assert_eq!(memory.growth_calls.get(), growth_calls);
    assert_eq!(*memory.backing.borrow(), before);

    memory.refuse_growth.set(true);
    let (map, result) = map.reserve_pages(129);
    assert!(matches!(
        result,
        Err(ic_memory::RuntimeGrowError::BackingRefused { .. })
    ));
    assert_eq!(map.get(&7), Some(11));
    assert_eq!(*memory.backing.borrow(), before);
    assert_eq!(runtime.memory_allocations().unwrap(), allocations);
    let (map, pages) = map.allocated_pages();
    assert_eq!(pages, 8);
    assert!(matches!(
        runtime.open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap()),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));

    memory.refuse_growth.set(false);
    let (mut map, result) = map.reserve_pages(129);
    result.unwrap();
    map.insert(8, 12);
    drop(map);
    drop(runtime);
    let runtime = self::runtime(memory);
    let map = StableBTreeMap::<u64, u64, _>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap())
            .unwrap(),
    );
    let (map, pages) = map.allocated_pages();
    assert_eq!(pages, 129);
    assert_eq!(map.get(&7), Some(11));
    assert_eq!(map.get(&8), Some(12));
}

use std::borrow::Cow;
#[derive(Clone, Debug, Eq, PartialEq)]
struct Record<const FIXED: bool = false>(Vec<u8>);

impl<const FIXED: bool> Storable for Record<FIXED> {
    const BOUND: Bound = Bound::Bounded {
        max_size: 4,
        is_fixed_size: FIXED,
    };
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
    fn into_bytes(self) -> Vec<u8> {
        self.0
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(bytes.into_owned())
    }
}

#[test]
fn bounded_cell_rejects_oversize_without_mutation_and_reopens() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let mut cell = StableCell::create(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
            .unwrap(),
        None::<Record>,
    );
    cell.set(Some(Record(vec![7; 4])));
    let before = backing.borrow().clone();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cell.set(Some(Record(vec![8; 5])));
    }));
    assert!(failed.is_err());
    assert_eq!(*backing.borrow(), before);
    assert_eq!(cell.get(), &Some(Record(vec![7; 4])));
    assert!(matches!(
        runtime.open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap()),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    drop(cell);
    drop(runtime);
    let runtime = self::runtime(backing);
    let mut cell = StableCell::<Option<Record>>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(cell.get(), &Some(Record(vec![7; 4])));
    cell.set(None);
    drop(cell);
    let cell = StableCell::<Option<Record>>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(cell.get().is_none());
}

#[test]
fn cell_creation_rejects_invalid_fixed_and_variable_encodings_before_writing() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let before = backing.borrow().clone();
    for fixed in [true, false] {
        let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let memory = runtime
                .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
                .unwrap();
            if fixed {
                let _ = StableCell::create(memory, Record::<true>(vec![1; 3]));
            } else {
                let _ = StableCell::create(memory, Record::<false>(vec![1; 5]));
            }
        }));
        assert!(rejected.is_err());
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn bounded_cell_rejects_short_and_long_fixed_replacements_before_writing() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let mut cell = StableCell::create(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
            .unwrap(),
        Record::<true>(vec![7; 4]),
    );
    let before = backing.borrow().clone();
    for length in [0, 3, 5] {
        let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.set(Record(vec![8; length]));
        }));
        assert!(rejected.is_err());
        assert_eq!(*backing.borrow(), before);
        assert_eq!(cell.get(), &Record(vec![7; 4]));
    }
    assert_eq!(cell.set(Record(vec![9; 4])), Record(vec![7; 4]));
    drop(cell);
    let reopened = StableCell::<Record<true>>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(reopened.get(), &Record(vec![9; 4]));
}

#[test]
fn bounded_cell_validates_retained_bytes_without_rewriting_or_leaking_lease() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    for fixed in [false, true] {
        for length in [0, 3, 4, 5] {
            let bytes = vec![7; length];
            // Produce existing upstream cell bytes through the supported adapter.
            let existing = StableCell::reset(
                runtime
                    .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
                    .unwrap(),
                bytes.clone(),
            );
            drop(existing);
            let before = backing.borrow().clone();
            let opened = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let memory = runtime
                    .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
                    .unwrap();
                if fixed {
                    let cell = StableCell::<Record<true>>::load(memory).unwrap();
                    assert_eq!(cell.get().0, bytes);
                } else {
                    let cell = StableCell::<Record<false>>::load(memory).unwrap();
                    assert_eq!(cell.get().0, bytes);
                }
            }));
            assert_eq!(
                opened.is_ok(),
                if fixed { length == 4 } else { length <= 4 }
            );
            assert_eq!(*backing.borrow(), before);
            let existing = StableCell::<Vec<u8>>::load(
                runtime
                    .open_memory(&ic_memory::StableKey::parse("test.cell.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(existing.get(), &bytes);
        }
    }
}

#[test]
fn map_sequences_match_model_across_splits_removals_and_reopens() {
    use std::collections::BTreeMap;
    let backing = VectorMemory::default();
    let mut model = BTreeMap::new();
    let mut state = 0x1C50_u64;
    for generation in 0..8 {
        let host = runtime(backing.clone());
        let memory = host
            .open_memory(&ic_memory::StableKey::parse("test.map.v1").unwrap())
            .unwrap();
        let mut map = if generation == 0 {
            StableBTreeMap::<u64, u64>::create(memory)
        } else {
            StableBTreeMap::<u64, u64>::load(memory)
        };
        for step in 0..500 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let key = (state >> 32) % 400;
            if step % 3 == 0 && generation > 0 {
                assert_eq!(map.remove(&key), model.remove(&key));
            } else {
                assert_eq!(map.insert(key, state), model.insert(key, state));
            }
            assert_eq!(map.get(&key), model.get(&key).copied());
        }
        assert_eq!(map.len(), model.len() as u64);
        assert_eq!(
            map.iter()
                .map(<MapIter<'_, u64, u64> as Iterator>::Item::into_pair)
                .collect::<Vec<_>>(),
            model
                .iter()
                .map(|(&key, &value)| (key, value))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            map.range(100..200)
                .map(<MapIter<'_, u64, u64> as Iterator>::Item::into_pair)
                .collect::<Vec<_>>(),
            model
                .range(100..200)
                .map(|(&key, &value)| (key, value))
                .collect::<Vec<_>>()
        );
        drop(map);
        drop(host);
    }
}
