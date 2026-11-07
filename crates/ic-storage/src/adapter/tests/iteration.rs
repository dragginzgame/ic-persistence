//! Terminal entry traversal respects the remaining bidirectional range.
use super::*;
use crate::testing::MeteredMemory;
use std::{fmt::Debug, ops::Bound as RangeBound};

fn remaining_last<I, T>(
    mut actual: I,
    expected: &[T],
    front: usize,
    back: usize,
    decode: impl Fn(I::Item) -> T,
) where
    I: DoubleEndedIterator,
    T: Clone + Debug + PartialEq,
{
    let mut expected = expected.iter().cloned();
    if front > 0 {
        assert_eq!(actual.nth(front - 1).map(&decode), expected.nth(front - 1));
    }
    if back > 0 {
        assert_eq!(
            actual.nth_back(back - 1).map(&decode),
            expected.nth_back(back - 1)
        );
    }
    assert_eq!(Iterator::last(actual).map(decode), expected.next_back());
}

fn terminal_history<K: Storable + Ord + Clone + Debug>(key: impl Fn(u64) -> K) {
    use RangeBound::{Excluded, Included, Unbounded};
    for population in [0, 1, 97] {
        let host = runtime(VectorMemory::default());
        let mut map = StableBTreeMap::<K, u64>::create(
            host.open_memory(&crate::StableKey::parse(KEYS[0]).unwrap())
                .unwrap(),
        )
        .unwrap();
        let mut set = StableBTreeSet::<K>::create(
            host.open_memory(&crate::StableKey::parse(KEYS[1]).unwrap())
                .unwrap(),
        )
        .unwrap();
        let mut model = std::collections::BTreeMap::new();
        for n in (0..population).rev() {
            map.insert(key(n), n * 3).unwrap();
            set.insert(key(n)).unwrap();
            model.insert(key(n), n * 3);
        }
        let pairs: Vec<_> = model.iter().map(|(k, v)| (k.clone(), *v)).collect();
        let keys: Vec<_> = model.keys().cloned().collect();
        for (front, back) in [(0, 0), (1, 0), (0, 1), (2, 3), (100, 0), (0, 100)] {
            remaining_last(map.iter(), &pairs, front, back, |e| {
                e.unwrap().into_pair().unwrap()
            });
            remaining_last(set.iter(), &keys, front, back, Result::unwrap);
            let reversed: Vec<_> = pairs.iter().rev().cloned().collect();
            remaining_last(map.iter().rev(), &reversed, front, back, |e| {
                e.unwrap().into_pair().unwrap()
            });
            for bounds in [
                (Unbounded, Included(key(90))),
                (Included(key(7)), Excluded(key(82))),
                (Excluded(key(7)), Included(key(82))),
                (Included(key(7)), Excluded(key(7))),
                (Included(key(200)), Unbounded),
            ] {
                let expected: Vec<_> = model
                    .range(bounds.clone())
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                remaining_last(map.range(bounds), &expected, front, back, |e| {
                    e.unwrap().into_pair().unwrap()
                });
            }
        }
    }
}

#[test]
fn last_preserves_empty_bounded_and_partially_consumed_results() {
    terminal_history(|n| n);
    terminal_history(|n| format!("{n:06}"));
    terminal_history(|n| format!("{n:06}{}", "x".repeat(4096)));
    terminal_history(u64::to_be_bytes);
}

#[test]
fn last_reads_a_tree_path_without_walking_the_population() {
    let backing = MeteredMemory::<VectorMemory>::default();
    let host = runtime(backing.clone());
    let mut map = StableBTreeMap::<u64, u64, _>::create(
        host.open_memory(&crate::StableKey::parse(KEYS[0]).unwrap())
            .unwrap(),
    )
    .unwrap();
    for n in 0..512 {
        map.insert(n, n * 3).unwrap();
    }
    backing.reset();
    for (n, entry) in map.iter().enumerate() {
        assert_eq!(
            entry.unwrap().into_pair().unwrap(),
            (n as u64, n as u64 * 3)
        );
    }
    let scan = backing.counts();
    backing.reset();
    assert_eq!(
        map.iter().last().unwrap().unwrap().into_pair().unwrap(),
        (511, 1533)
    );
    let last = backing.counts();
    assert!(!scan.overflowed && !last.overflowed);
    assert_eq!((last.writes, last.grows), (0, 0));
    assert!(last.reads * 8 < scan.reads, "last={last:?}, scan={scan:?}");
    assert!(
        last.read_bytes * 8 < scan.read_bytes,
        "last={last:?}, scan={scan:?}"
    );
}
