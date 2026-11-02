use super::*;
use crate::{StableKey, StorageConfig, VectorMemory, testing::fixture_runtime};
use std::{borrow::Cow, cell::Cell, cmp::Ordering};

fn runtime(backing: VectorMemory) -> crate::MemoryRuntime<VectorMemory> {
    fixture_runtime(
        backing,
        "test",
        "test.",
        &["test.cell.v1", "test.vector.v1", "test.map.v1"],
        StorageConfig::default(),
    )
    .unwrap()
}

#[test]
fn bounded_map_keys_skip_external_values_and_support_both_directions() {
    let backing = crate::testing::MeteredMemory::<VectorMemory>::default();
    let runtime = fixture_runtime(
        backing.clone(),
        "test",
        "test.",
        &["test.map.v1"],
        StorageConfig::default(),
    )
    .unwrap();
    let mut map = StableBTreeMap::<u64, Vec<u8>, _>::create(
        runtime
            .open_memory(&StableKey::parse("test.map.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    for key in 0..40 {
        map.insert(key, vec![3; 16_384]).unwrap();
    }
    backing.reset();
    let mut keys = map.keys_range(10..14);
    assert_eq!(keys.next().unwrap().unwrap(), 10);
    assert_eq!(keys.next_back().unwrap().unwrap(), 13);
    assert_eq!(keys.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 12]);
    assert!(backing.counts().read_bytes < 16_384);
    assert_eq!(backing.counts().writes, 0);
}

#[test]
fn fixed_vector_reads_once_per_slot_and_reuses_iteration_buffer() {
    let backing = crate::testing::MeteredMemory::<VectorMemory>::default();
    let runtime = fixture_runtime(
        backing.clone(),
        "test",
        "test.",
        &["test.vector.v1"],
        StorageConfig::default(),
    )
    .unwrap();
    let vector = StableVec::<u64, _>::create(
        runtime
            .open_memory(&StableKey::parse("test.vector.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    for value in 0..20 {
        vector.push(&value).unwrap();
    }
    backing.reset();
    assert_eq!(vector.get(7).unwrap(), Some(7));
    assert_eq!(backing.counts().reads, 1);
    assert_eq!(backing.counts().read_bytes, 20);
    backing.reset();
    let mut iter = vector.iter();
    assert_eq!(iter.next().unwrap().unwrap(), 0);
    assert_eq!(iter.next_back().unwrap().unwrap(), 19);
    assert_eq!(
        iter.collect::<Result<Vec<_>, _>>().unwrap(),
        (1..19).collect::<Vec<_>>()
    );
    assert_eq!(backing.counts().reads, 20);
}

#[test]
fn cell_budget_and_checksum_rejections_are_read_only_and_release_leases() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let key = StableKey::parse("test.cell.v1").unwrap();
    drop(StableCell::create(runtime.open_memory(&key).unwrap(), vec![7_u8; 32]).unwrap());
    let before = backing.borrow().clone();
    assert!(matches!(
        StableCell::<Vec<u8>>::load(runtime.open_memory(&key).unwrap(), 31),
        Err(CollectionError::Cell(
            crate::StableCellPayloadError::TooLarge { .. }
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    let lease = runtime.open_memory(&key).unwrap();
    lease.write(crate::STABLE_CELL_HEADER_SIZE as u64, &[9]);
    drop(lease);
    let corrupt = backing.borrow().clone();
    assert!(matches!(
        StableCell::<Vec<u8>>::load(runtime.open_memory(&key).unwrap(), 32),
        Err(CollectionError::Checksum)
    ));
    assert_eq!(*backing.borrow(), corrupt);
    assert!(runtime.open_memory(&key).is_ok());
}

#[test]
fn vector_payload_and_slot_relocation_are_detected_without_mutation() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let key = StableKey::parse("test.vector.v1").unwrap();
    let vector = StableVec::<u64>::create(runtime.open_memory(&key).unwrap()).unwrap();
    vector.push(&11).unwrap();
    vector.push(&22).unwrap();
    drop(vector);
    let lease = runtime.open_memory(&key).unwrap();
    let mut slot = [0; 20];
    lease.read(record::HEADER_BYTES, &mut slot);
    lease.write(record::HEADER_BYTES + 20, &slot);
    lease.write(record::HEADER_BYTES + 12, &[99]);
    drop(lease);
    let before = backing.borrow().clone();
    let vector = StableVec::<u64>::load(runtime.open_memory(&key).unwrap()).unwrap();
    for index in 0..2 {
        assert!(matches!(vector.get(index), Err(CollectionError::Checksum)));
    }
    assert_eq!(*backing.borrow(), before);
}

thread_local! { static COMPARISONS: Cell<usize> = const { Cell::new(0) }; }
#[derive(Clone, Eq, PartialEq)]
struct Counted(u64);
impl Ord for Counted {
    fn cmp(&self, other: &Self) -> Ordering {
        COMPARISONS.with(|count| count.set(count.get() + 1));
        self.0.cmp(&other.0)
    }
}
impl PartialOrd for Counted {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Storable for Counted {
    const BOUND: Bound = u64::BOUND;
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        self.0.to_bytes()
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        u64::from_bytes(bytes).map(Self)
    }
}

#[test]
fn deletion_extracts_successor_without_a_second_key_search() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing);
    let key = StableKey::parse("test.map.v1").unwrap();
    let mut map =
        StableBTreeMap::<Counted, u64>::create(runtime.open_memory(&key).unwrap()).unwrap();
    for key in 0..255 {
        map.insert(Counted(key), key).unwrap();
    }
    COMPARISONS.with(|count| count.set(0));
    assert_eq!(map.get(&Counted(6)).unwrap(), Some(6));
    let lookup_comparisons = COMPARISONS.with(Cell::get);
    COMPARISONS.with(|count| count.set(0));
    assert_eq!(map.remove(&Counted(6)).unwrap(), Some(6));
    assert_eq!(COMPARISONS.with(Cell::get), lookup_comparisons);
    drop(map);
    let map = StableBTreeMap::<Counted, u64>::load(runtime.open_memory(&key).unwrap()).unwrap();
    assert_eq!(
        map.values().collect::<Result<Vec<_>, _>>().unwrap(),
        (0..255).filter(|key| *key != 6).collect::<Vec<_>>()
    );
}

#[derive(Clone)]
struct Rejected(u8);
impl Storable for Rejected {
    const BOUND: Bound = u8::BOUND;
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        if self.0 == 0 {
            Err(CodecError::Invalid("encoding refused"))
        } else {
            self.0.to_bytes()
        }
    }
    fn from_bytes(_: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        Err(CodecError::Invalid("decoding refused"))
    }
}

#[test]
fn codec_errors_and_previous_value_decode_failure_do_not_commit_map_changes() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let key = StableKey::parse("test.map.v1").unwrap();
    let mut map =
        StableBTreeMap::<u64, Rejected>::create(runtime.open_memory(&key).unwrap()).unwrap();
    map.insert(1, Rejected(1)).unwrap();
    let before = backing.borrow().clone();
    for result in [
        map.insert(2, Rejected(0)),
        map.insert(1, Rejected(2)),
        map.remove(&1),
        map.get(&1),
    ] {
        assert!(matches!(result, Err(CollectionError::Codec(_))));
        assert_eq!(*backing.borrow(), before);
    }
    assert_eq!(map.len(), 1);
    assert!(matches!(
        bool::from_bytes(Cow::Borrowed(&[2])),
        Err(CodecError::Invalid(_))
    ));
    assert!(matches!(
        String::from_bytes(Cow::Borrowed(&[255])),
        Err(CodecError::Invalid(_))
    ));
}

#[test]
fn map_iteration_returns_one_error_then_fuses_in_both_directions() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let key = StableKey::parse("test.map.v1").unwrap();
    let mut map = StableBTreeMap::<u64, u64>::create(runtime.open_memory(&key).unwrap()).unwrap();
    map.insert(1, 2).unwrap();
    let mut invalid = map.range((std::ops::Bound::Excluded(1), std::ops::Bound::Excluded(1)));
    assert!(matches!(invalid.next(), Some(Err(CollectionError::Bounds))));
    assert!(invalid.next_back().is_none());
    drop(invalid);
    drop(map);
    let lease = runtime.open_memory(&key).unwrap();
    lease.write(record::HEADER_BYTES, b"CORRUPT!");
    drop(lease);
    let before = backing.borrow().clone();
    let map = StableBTreeMap::<u64, u64>::load(runtime.open_memory(&key).unwrap()).unwrap();
    for reverse in [false, true] {
        let mut iter = map.iter();
        let result = if reverse {
            iter.next_back()
        } else {
            iter.next()
        };
        assert!(matches!(result, Some(Err(CollectionError::Format))));
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }
    assert_eq!(*backing.borrow(), before);
}
