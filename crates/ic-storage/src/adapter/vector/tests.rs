//! Population-scaled IO and the scalar ICSS iterator contract.
use super::*;
use crate::{
    StableKey, StorageConfig,
    testing::{MeteredMemory, fixture_runtime},
};
use std::{
    borrow::Cow,
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
thread_local! {static DECODES:Cell<(u64,usize)>=const{Cell::new((0,0))};}
struct Record<const MAXIMUM: u32 = 4096, const FIXED: bool = false>(Vec<u8>);
impl<const MAXIMUM: u32, const FIXED: bool> Storable for Record<MAXIMUM, FIXED> {
    const BOUND: super::super::Bound = super::super::Bound::Bounded {
        max_size: MAXIMUM,
        is_fixed_size: FIXED,
    };
    fn to_bytes(&self) -> std::result::Result<Cow<'_, [u8]>, super::super::CodecError> {
        Ok(Cow::Borrowed(&self.0))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> std::result::Result<Self, super::super::CodecError> {
        DECODES.with(|x| {
            let (c, n) = x.get();
            x.set((c + 1, n + bytes.len()));
        });
        Ok(Self(bytes.into_owned()))
    }
}
fn runtime<M: crate::MemoryBacking>(backing: M) -> crate::MemoryRuntime<M> {
    fixture_runtime(
        backing,
        "test",
        "test.",
        &["test.vector.v1"],
        StorageConfig::default(),
    )
    .unwrap()
}
fn key() -> StableKey {
    StableKey::parse("test.vector.v1").unwrap()
}
#[test]
fn populated_lookups_read_one_length_one_slot_and_decode_once_after_reopen() {
    for payload in [256, 2048] {
        for population in [1u64, 512, 1024, 2048] {
            let backing = MeteredMemory::<crate::VectorMemory>::default();
            let runtime = runtime(backing.clone());
            let vector =
                StableVec::<Record, _>::create(runtime.open_memory(&key()).unwrap()).unwrap();
            for id in 0..population {
                let mut bytes = vec![42; payload];
                bytes[..8].copy_from_slice(&id.to_le_bytes());
                vector.push(&Record(bytes)).unwrap();
            }
            drop(vector);
            let vector =
                StableVec::<Record, _>::load(runtime.open_memory(&key()).unwrap()).unwrap();
            for index in [0, population / 2, population - 1] {
                backing.reset();
                DECODES.with(|x| x.set((0, 0)));
                let item = vector.get(index).unwrap().unwrap();
                assert_eq!(&item.0[..8], &index.to_le_bytes());
                let io = backing.counts();
                assert!(!io.overflowed);
                assert!((3..=5).contains(&io.reads));
                if index == 0 {
                    assert_eq!(io.reads, 3);
                }
                assert_eq!(io.read_bytes, payload as u64 + 2 + 8);
                assert_eq!((io.writes, io.grows), (0, 0));
                DECODES.with(|x| assert_eq!(x.get(), (1, payload)));
            }
            backing.reset();
            DECODES.with(|x| x.set((0, 0)));
            assert!(vector.get(population).unwrap().is_none());
            assert_eq!(
                (backing.counts().reads, backing.counts().read_bytes),
                (1, 8)
            );
            DECODES.with(|x| assert_eq!(x.get(), (0, 0)));
        }
    }
}
fn check_geometry<const MAXIMUM: u32, const FIXED: bool>(prefix: u64) {
    let backing = MeteredMemory::<crate::VectorMemory>::default();
    let runtime = runtime(backing.clone());
    let vector =
        StableVec::<Record<MAXIMUM, FIXED>, _>::create(runtime.open_memory(&key()).unwrap())
            .unwrap();
    let sizes = if FIXED {
        vec![MAXIMUM as usize; 3]
    } else {
        vec![0, MAXIMUM.min(32) as usize, MAXIMUM as usize]
    };
    for (i, &n) in sizes.iter().enumerate() {
        vector
            .push(&Record(vec![u8::try_from(i).unwrap(); n]))
            .unwrap();
    }
    for (i, &n) in sizes.iter().enumerate() {
        backing.reset();
        assert_eq!(
            vector.get(i as u64).unwrap().unwrap().0,
            vec![u8::try_from(i).unwrap(); n]
        );
        assert_eq!(backing.counts().read_bytes, n as u64 + prefix + 8);
    }
    assert_eq!(vector.pop().unwrap().unwrap().0, vec![2; sizes[2]]);
    drop(vector);
    let vector =
        StableVec::<Record<MAXIMUM, FIXED>, _>::load(runtime.open_memory(&key()).unwrap()).unwrap();
    assert_eq!(vector.len(), 2);
}
#[test]
fn scalar_geometry_covers_length_widths_and_zero_sized_fixed_records() {
    check_geometry::<0, false>(1);
    check_geometry::<255, false>(1);
    check_geometry::<256, false>(2);
    check_geometry::<65535, false>(2);
    check_geometry::<65536, false>(4);
    check_geometry::<0, true>(0);
    check_geometry::<32, true>(0);
}
#[test]
fn iterators_see_writes_and_match_upstream_truncation_and_skipping() {
    let runtime = runtime(crate::VectorMemory::default());
    let vector = StableVec::<u64, _>::create(runtime.open_memory(&key()).unwrap()).unwrap();
    for value in 0..20 {
        vector.push(&value).unwrap();
    }
    let mut iter = vector.iter();
    assert_eq!(iter.next().unwrap().unwrap(), 0);
    vector.set(1, &99).unwrap();
    assert_eq!(iter.next().unwrap().unwrap(), 99);
    assert_eq!(vector.pop().unwrap(), Some(19));
    assert!(iter.next_back().is_none());
    vector.push(&77).unwrap();
    assert_eq!(iter.next_back().unwrap().unwrap(), 77);
    assert_eq!(iter.nth(2).unwrap().unwrap(), 4);
    assert_eq!(iter.nth_back(2).unwrap().unwrap(), 16);
    assert_eq!(iter.last().unwrap().unwrap(), 15);
    assert_eq!(
        vector.iter().rev().collect::<Result<Vec<_>>>().unwrap(),
        [vec![77], (2..19).rev().collect(), vec![99, 0]].concat()
    );
}
#[test]
fn iterator_skips_and_count_delegate_without_decoding_discarded_records() {
    let backing = MeteredMemory::<crate::VectorMemory>::default();
    let runtime = runtime(backing.clone());
    let vector = StableVec::<Record, _>::create(runtime.open_memory(&key()).unwrap()).unwrap();
    for id in 0u64..256 {
        let mut bytes = vec![42; 256];
        bytes[..8].copy_from_slice(&id.to_le_bytes());
        vector.push(&Record(bytes)).unwrap();
    }
    for reverse in [false, true] {
        backing.reset();
        DECODES.with(|x| x.set((0, 0)));
        let mut iter = vector.iter();
        let item = if reverse {
            iter.nth_back(254)
        } else {
            iter.nth(254)
        };
        let expected = if reverse { 1u64 } else { 254u64 };
        assert_eq!(&item.unwrap().unwrap().0[..8], &expected.to_le_bytes());
        DECODES.with(|x| assert_eq!(x.get(), (1, 256)));
        assert_eq!(backing.counts().read_bytes, 8 + 8 + 2 + 256);
        assert_eq!((backing.counts().writes, backing.counts().grows), (0, 0));
    }
    backing.reset();
    DECODES.with(|x| x.set((0, 0)));
    assert_eq!(vector.iter().count(), 256);
    DECODES.with(|x| assert_eq!(x.get(), (0, 0)));
    assert_eq!(backing.counts().read_bytes, 16);
}
#[test]
fn malformed_slot_length_traps_before_payload_allocation_or_decode() {
    let backing = MeteredMemory::<crate::VectorMemory>::default();
    let runtime = runtime(backing.clone());
    let vector = StableVec::<Record, _>::create(runtime.open_memory(&key()).unwrap()).unwrap();
    vector.push(&Record(vec![7; 256])).unwrap();
    vector.inner.memory().0.write(64, &4097u16.to_le_bytes());
    backing.reset();
    DECODES.with(|x| x.set((0, 0)));
    assert!(catch_unwind(AssertUnwindSafe(|| vector.get(0))).is_err());
    assert_eq!(backing.counts().read_bytes, 10);
    assert_eq!(backing.counts().writes, 0);
    DECODES.with(|x| assert_eq!(x.get(), (0, 0)));
}
