//! Qualification of the owned collection formats and staged mutation behavior.
use super::*;
use crate::VectorMemory;
use std::{
    cell::Cell,
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

#[test]
fn vector_model_mixed_updates_reopen_and_bidirectional_iteration() {
    let backing = VectorMemory::default();
    let mut expected = Vec::new();
    for generation in 0..6 {
        let host = super::tests::runtime(backing.clone());
        let memory = host
            .open_memory(&crate::StableKey::parse("test.vec.v1").unwrap())
            .unwrap();
        let vector = if generation == 0 {
            StableVec::<u64>::create(memory).unwrap()
        } else {
            StableVec::load(memory).unwrap()
        };
        for i in 0..250 {
            let value = generation * 1000 + i;
            vector.push(&value).unwrap();
            expected.push(value);
            if i % 3 == 0 {
                assert_eq!(vector.pop().unwrap(), expected.pop());
            }
            if i % 7 == 0 && !expected.is_empty() {
                vector.set(0, &value).unwrap();
                expected[0] = value;
            }
        }
        assert_eq!(
            vector.iter().collect::<Result<Vec<_>, _>>().unwrap(),
            expected
        );
        assert_eq!(
            vector.iter().rev().collect::<Result<Vec<_>, _>>().unwrap(),
            expected.iter().copied().rev().collect::<Vec<_>>()
        );
        assert_eq!(vector.get(u64::MAX).unwrap(), None);
        let before = backing.borrow().clone();
        assert!(vector.set(vector.len(), &7).is_err());
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn ordered_tree_mixed_direction_ranges_reuse_storage_and_clear() {
    let backing = VectorMemory::default();
    let mut tree = tree::Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    let mut expected = BTreeMap::new();
    for key in (0..500).rev() {
        tree.insert(key, vec![9; 25]).unwrap();
        expected.insert(key, vec![9; 25]);
    }
    for key in (0..500).step_by(2) {
        assert_eq!(tree.remove(&key).unwrap(), expected.remove(&key));
    }
    let range = tree.range(100..200);
    assert_eq!(
        range
            .map(|entry| entry.unwrap().into_pair().unwrap())
            .collect::<Vec<_>>(),
        expected
            .range(100..200)
            .map(|(k, v)| (*k, v.clone()))
            .collect::<Vec<_>>()
    );
    let mut range = tree.range(100..108);
    assert_eq!(range.next().unwrap().unwrap().into_key(), 101);
    assert_eq!(range.next_back().unwrap().unwrap().into_key(), 107);
    assert_eq!(range.next_back().unwrap().unwrap().into_key(), 105);
    assert_eq!(range.next().unwrap().unwrap().into_key(), 103);
    assert!(range.next().is_none());
    assert!(range.next_back().is_none());
    let pages = backing.size();
    for _ in 0..4 {
        for key in (0..500).step_by(2) {
            tree.insert(key, vec![8; 25]).unwrap();
        }
        for key in (0..500).step_by(2) {
            tree.remove(&key).unwrap();
        }
        assert_eq!(backing.size(), pages, "free blocks must be reused");
    }
    tree.clear().unwrap();
    assert_eq!(tree.len(), 0);
    for key in 0..500 {
        tree.insert(key, vec![7; 25]).unwrap();
    }
    assert_eq!(backing.size(), pages);
    drop(tree);
    let tree = tree::Tree::<u64, Vec<u8>, _>::load(backing).unwrap();
    assert_eq!(tree.len(), 500);
    assert!(
        tree.range(..)
            .map(|entry| entry.unwrap().into_key())
            .is_sorted()
    );
}

#[derive(Clone, Default)]
struct FaultMemory {
    memory: VectorMemory,
    refuse: Rc<Cell<bool>>,
    fail_write: Rc<Cell<bool>>,
    reads: Rc<Cell<usize>>,
    read_bytes: Rc<Cell<usize>>,
    writes: Rc<Cell<usize>>,
}
impl Memory for FaultMemory {
    fn size(&self) -> u64 {
        self.memory.size()
    }
    fn grow(&self, delta: u64) -> i64 {
        if self.refuse.get() {
            -1
        } else {
            self.memory.grow(delta)
        }
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.reads.set(self.reads.get() + 1);
        self.read_bytes.set(self.read_bytes.get() + dst.len());
        self.memory.read(offset, dst);
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        self.writes.set(self.writes.get() + 1);
        assert!(!self.fail_write.get(), "injected write failure");
        self.memory.write(offset, bytes);
    }
}

#[test]
fn replacement_and_scans_have_bounded_backing_traffic() {
    let backing = FaultMemory::default();
    let mut tree = tree::Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    for key in 0..255 {
        tree.insert(key, vec![1; 32]).unwrap();
    }
    backing.writes.set(0);
    assert_eq!(tree.insert(0, vec![2; 32]).unwrap(), Some(vec![1; 32]));
    // In-place replacement does not rewrite unchanged root/allocation metadata.
    assert_eq!(backing.writes.get(), 1);
    assert_eq!(tree.len(), 255);
    for reverse in [false, true] {
        backing.reads.set(0);
        let keys: Vec<_> = if reverse {
            tree.range(..)
                .rev()
                .map(|entry| entry.unwrap().into_key())
                .collect()
        } else {
            tree.range(..)
                .map(|entry| entry.unwrap().into_key())
                .collect()
        };
        let mut expected: Vec<_> = (0..255).collect();
        if reverse {
            expected.reverse();
        }
        assert_eq!(keys, expected);
        // Linear traversal budget, including prefix/payload reads and spine setup.
        assert!(backing.reads.get() <= 5 * 255);
    }
    drop(tree);
    let tree = tree::Tree::<u64, Vec<u8>, _>::load(backing).unwrap();
    assert_eq!(tree.get(&0).unwrap(), Some(vec![2; 32]));
}

#[test]
fn cursor_bounds_and_meeting_match_the_ordered_model() {
    use std::ops::Bound::{Excluded, Included};
    let mut tree = tree::Tree::<u64, u64, _>::reset(VectorMemory::default()).unwrap();
    let mut model = BTreeMap::new();
    for key in (0..20).step_by(2) {
        tree.insert(key, key).unwrap();
        model.insert(key, key);
    }
    for low in 0..20 {
        for high in low..20 {
            for lower in [Included(low), Excluded(low)] {
                for upper in [Included(high), Excluded(high)] {
                    if low == high && matches!((lower, upper), (Excluded(_), Excluded(_))) {
                        continue;
                    }
                    let mut actual = tree.range((lower, upper));
                    let mut expected = model.range((lower, upper));
                    for step in 0..22 {
                        let (a, b) = if step % 3 == 0 {
                            (actual.next_back(), expected.next_back())
                        } else {
                            (actual.next(), expected.next())
                        };
                        assert_eq!(
                            a.map(|entry| entry.unwrap().into_pair().unwrap()),
                            b.map(|(&k, &v)| (k, v))
                        );
                    }
                    assert!(actual.next().is_none() && actual.next_back().is_none());
                }
            }
        }
    }
}

#[test]
fn tree_growth_refusal_preserves_values_bytes_and_allocator_then_retries() {
    let backing = FaultMemory::default();
    let mut tree = tree::Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    tree.insert(7, vec![1]).unwrap();
    let before = backing.memory.borrow().clone();
    backing.refuse.set(true);
    assert!(tree.insert(7, vec![2; 100_000]).is_err());
    assert_eq!(*backing.memory.borrow(), before);
    assert_eq!(tree.get(&7).unwrap(), Some(vec![1]));
    backing.refuse.set(false);
    assert_eq!(tree.insert(7, vec![2; 100_000]).unwrap(), Some(vec![1]));
    backing.fail_write.set(true);
    assert!(catch_unwind(AssertUnwindSafe(|| tree.insert(9, vec![3]).unwrap())).is_err());
    backing.fail_write.set(false);
    assert!(tree.get(&7).is_err());
}

#[test]
fn owned_root_corruption_and_foreign_formats_never_initialize_or_write() {
    for offset in [0, 7, 8, 12, 20] {
        let backing = VectorMemory::default();
        drop(tree::Tree::<u64, u64, _>::reset(backing.clone()).unwrap());
        backing.borrow_mut()[offset] ^= 1;
        let before = backing.borrow().clone();
        assert!(tree::Tree::<u64, u64, _>::load(backing.clone()).is_err());
        assert_eq!(*backing.borrow(), before);
    }
    let backing = VectorMemory::default();
    backing.grow(1);
    backing.write(0, b"foreign!");
    let before = backing.borrow().clone();
    assert!(tree::Tree::<u64, u64, _>::load(backing.clone()).is_err());
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn tree_variable_records_match_model_across_relocation_removal_and_reopen() {
    let backing = VectorMemory::default();
    let mut tree = tree::Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    let mut expected = BTreeMap::new();
    let mut seed = 7_u64;
    for step in 0..2000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let key = (seed >> 32) % 127;
        if step % 4 == 0 {
            assert_eq!(tree.remove(&key).unwrap(), expected.remove(&key));
        } else {
            let length = [0, 17, 240, 1024, 5000][usize::try_from(seed % 5).unwrap()];
            let value = vec![seed.to_le_bytes()[0]; length];
            assert_eq!(
                tree.insert(key, value.clone()).unwrap(),
                expected.insert(key, value)
            );
        }
        if step % 100 == 0 {
            drop(tree);
            tree = tree::Tree::load(backing.clone()).unwrap();
            assert_eq!(tree.len(), expected.len() as u64);
            assert_eq!(
                tree.range(..)
                    .map(|entry| entry.unwrap().into_pair().unwrap())
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|(key, value)| (*key, value.clone()))
                    .collect::<Vec<_>>()
            );
        }
    }
    for (key, value) in expected {
        assert_eq!(tree.remove(&key).unwrap(), Some(value));
    }
    assert_eq!(tree.len(), 0);
    drop(tree);
    assert_eq!(
        tree::Tree::<u64, Vec<u8>, _>::load(backing).unwrap().len(),
        0
    );
}

#[test]
fn routing_and_key_scans_do_not_read_external_value_payloads() {
    let backing = FaultMemory::default();
    let mut tree = tree::Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    for key in 0..200 {
        tree.insert(key, vec![u8::try_from(key).unwrap(); 16_384])
            .unwrap();
    }
    backing.read_bytes.set(0);
    assert!(!tree.contains_key(&201).unwrap());
    assert!(backing.read_bytes.get() < 8192);
    backing.read_bytes.set(0);
    assert_eq!(
        tree.keys_range(..).collect::<Result<Vec<_>, _>>().unwrap(),
        (0..200).collect::<Vec<_>>()
    );
    assert!(backing.read_bytes.get() < 32_768);
    backing.read_bytes.set(0);
    assert_eq!(tree.get(&199).unwrap(), Some(vec![199; 16_384]));
    assert!(backing.read_bytes.get() < 24_576);
}

#[test]
fn btree_variable_keys_split_merge_and_collapse_match_model() {
    let backing = VectorMemory::default();
    let mut tree = tree::Tree::<String, Vec<u8>, _>::reset(backing.clone()).unwrap();
    let mut model = BTreeMap::new();
    for pass in 0..3 {
        for id in (0..1500).map(|i| (i * 997) % 1500) {
            let key = format!("{id:05}/{}", "k".repeat(id % 129));
            let value = vec![pass; [0, 8, 32, 33, 1024][id % 5]];
            assert_eq!(
                tree.insert(key.clone(), value.clone()).unwrap(),
                model.insert(key, value)
            );
        }
        drop(tree);
        tree = tree::Tree::load(backing.clone()).unwrap();
        assert_eq!(
            tree.range(..)
                .map(|e| e.unwrap().into_pair().unwrap())
                .collect::<Vec<_>>(),
            model
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>()
        );
        for id in (0..1500).map(|i| (i * 991) % 1500) {
            let key = format!("{id:05}/{}", "k".repeat(id % 129));
            assert_eq!(tree.remove(&key).unwrap(), model.remove(&key));
            if id % 127 == 0 {
                drop(tree);
                tree = tree::Tree::load(backing.clone()).unwrap();
            }
        }
        assert_eq!(tree.len(), 0);
        assert!(tree.range(..).next().is_none());
    }
}
