use super::*;
use crate::VectorMemory;
use std::collections::BTreeSet;
mod generated;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn routing_cache_keeps_repeated_right_edge_misses_in_heap() {
    let memory = crate::testing::MeteredMemory::<VectorMemory>::default();
    let mut tree = Tree::<u64, u64, _>::reset(memory.clone()).unwrap();
    let mut keys: Vec<_> = (0..4096).collect();
    let mut seed = 1_u64;
    for end in (1..keys.len()).rev() {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        keys.swap(end, usize::try_from(seed % (end as u64 + 1)).unwrap());
    }
    for key in keys {
        tree.insert(key, key).unwrap();
    }
    tree.set_cache_budget(128 * 1024);
    assert_eq!(tree.get(&4096).unwrap(), None);
    memory.reset();
    for _ in 0..32 {
        assert_eq!(tree.get(&4096).unwrap(), None);
    }
    assert_eq!(memory.counts().reads, 0);
    eprintln!(
        "right-edge cache diagnostics: {:?}",
        tree.cache.borrow().metrics
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn routing_cache_reuses_reads_and_invalidates_mutations_and_clear() {
    let backing = crate::testing::MeteredMemory::<VectorMemory>::default();
    let mut tree = Tree::<u64, u64, _>::reset(backing.clone()).unwrap();
    for key in 0..1000 {
        tree.insert(key, key).unwrap();
    }
    tree.set_cache_budget(128 * 1024);
    assert_eq!(tree.get(&400).unwrap(), Some(400));
    backing.reset();
    assert_eq!(tree.get(&400).unwrap(), Some(400));
    assert_eq!(backing.counts().reads, 0);
    tree.insert(400, 999).unwrap();
    assert_eq!(tree.get(&400).unwrap(), Some(999));
    tree.remove(&400).unwrap();
    assert_eq!(tree.get(&400).unwrap(), None);
    tree.clear().unwrap();
    for key in 0..40 {
        tree.insert(key, key + 1).unwrap();
    }
    assert_eq!(tree.get(&10).unwrap(), Some(11));
    tree.set_cache_budget(0);
    backing.reset();
    assert_eq!(tree.get(&10).unwrap(), Some(11));
    assert!(backing.counts().reads > 0);
    drop(tree);
    let tree = Tree::<u64, u64, _>::load(backing).unwrap();
    assert_eq!(tree.get(&10).unwrap(), Some(11));
}

#[test]
fn mutations_without_structural_changes_do_not_stage_ancestors() {
    let backing = VectorMemory::default();
    let mut tree = Tree::<u64, u64, _>::reset(backing).unwrap();
    for key in 0..100 {
        tree.insert(key * 2, key).unwrap();
    }
    let root = tree.root.borrow().top.unwrap();
    read::OWNED_NODES.set(0);
    let mut tx = Transaction::new(&tree);
    let (top, split, old) = tx
        .insert(
            Some(root),
            &199,
            199_u64.to_le_bytes().to_vec(),
            1_u64.to_le_bytes().to_vec(),
            0,
        )
        .unwrap();
    assert_eq!(top, root);
    assert!(split.is_none() && old.is_none());
    assert!(!tx.writes.contains_key(&root.offset));
    assert_eq!(
        read::OWNED_NODES.get(),
        1,
        "only the inserted leaf is copied"
    );
    tx.root.top = Some(top);
    tx.commit().unwrap();
    read::OWNED_NODES.set(0);
    let mut tx = Transaction::new(&tree);
    let (_, old, _) = tx.remove(root, &199, 0).unwrap();
    assert_eq!(decode::<u64>(old.unwrap()).unwrap(), 1);
    assert!(!tx.writes.contains_key(&root.offset));
    assert_eq!(
        read::OWNED_NODES.get(),
        1,
        "no ancestor or rebalance probe copies"
    );
    // Root length publication is still required even without routing changes.
    tx.root.length -= 1;
    tx.commit().unwrap();
    assert_eq!(tree.len(), 100);
    assert_eq!(tree.get(&199).unwrap(), None);
    read::OWNED_NODES.set(0);
    assert_eq!(tree.remove(&199).unwrap(), None);
    assert_eq!(
        read::OWNED_NODES.get(),
        0,
        "missing removal is read-only traversal"
    );
}

#[test]
fn shrinking_live_values_reclaims_capacity_and_survives_reopen() {
    let backing = VectorMemory::default();
    let mut tree = Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    tree.insert(1, vec![7; 4096]).unwrap();
    tree.insert(2, vec![3; 80]).unwrap();
    tree.remove(&2).unwrap();
    let Value::External(original) = tree.find(&1).unwrap().unwrap() else {
        panic!()
    };
    let cursor = tree.root.borrow().cursor;
    assert_eq!(tree.insert(1, vec![8; 80]).unwrap(), Some(vec![7; 4096]));
    let Value::External(smaller) = tree.find(&1).unwrap().unwrap() else {
        panic!()
    };
    assert_ne!(smaller.offset, original.offset);
    assert_eq!(smaller.capacity(), 256);
    drop(tree);
    let mut tree = Tree::<u64, Vec<u8>, _>::load(backing).unwrap();
    tree.insert(2, vec![9; 1024]).unwrap();
    assert_eq!(tree.root.borrow().cursor, cursor);
    assert_eq!(tree.get(&1).unwrap(), Some(vec![8; 80]));
    assert_eq!(tree.get(&2).unwrap(), Some(vec![9; 1024]));
    check(&tree);
}

#[test]
fn alternating_large_and_small_values_does_not_consume_new_capacity() {
    for available_small_block in [false, true] {
        let memory = VectorMemory::default();
        let mut tree = Tree::<u64, Vec<u8>, _>::reset(memory.clone()).unwrap();
        tree.insert(1, vec![1; 4096]).unwrap();
        if available_small_block {
            tree.insert(2, vec![2; 80]).unwrap();
            tree.remove(&2).unwrap();
        }
        let cursor = tree.root.borrow().cursor;
        for step in 0..100 {
            let value = vec![3; if step % 2 == 0 { 80 } else { 4096 }];
            tree.insert(1, value.clone()).unwrap();
            assert_eq!(tree.get(&1).unwrap(), Some(value));
            assert_eq!(tree.root.borrow().cursor, cursor);
            drop(tree);
            tree = Tree::load(memory.clone()).unwrap();
            check(&tree);
        }
    }
}

#[test]
fn bounded_scans_do_not_read_excluded_values_in_either_direction() {
    let backing = VectorMemory::default();
    let mut tree = Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    for key in 0..3 {
        tree.insert(key, vec![u8::try_from(key).unwrap(); 80])
            .unwrap();
    }
    let Value::External(p) = tree.find(&1).unwrap().unwrap() else {
        panic!()
    };
    backing.borrow_mut()[usize::try_from(p.offset).unwrap() + 12] ^= 1;
    let before = backing.borrow().clone();
    let mut forward = tree.range(..1);
    assert_eq!(
        forward.next().unwrap().unwrap().into_pair().unwrap(),
        (0, vec![0; 80])
    );
    assert!(forward.next().is_none());
    let mut backward = tree.range((Bound::Excluded(1), Bound::Unbounded));
    assert_eq!(
        backward.next_back().unwrap().unwrap().into_pair().unwrap(),
        (2, vec![2; 80])
    );
    assert!(backward.next_back().is_none());
    assert!(matches!(tree.get(&1), Err(Error::Checksum)));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn a_single_entry_scan_does_not_prefetch_the_following_subtree() {
    for forward in [true, false] {
        let backing = VectorMemory::default();
        let mut tree = Tree::<u64, u64, _>::reset(backing.clone()).unwrap();
        for key in 0..100 {
            tree.insert(key, key).unwrap();
        }
        let root = tree.node(tree.root.borrow().top.unwrap()).unwrap();
        let index = root.entries.len() / 2;
        let key = decode::<u64>(root.entries[index].key.clone()).unwrap();
        let child = root.children[if forward { index + 1 } else { index }];
        backing.borrow_mut()[usize::try_from(child.offset).unwrap() + 12] ^= 1;
        let before = backing.borrow().clone();
        let mut scan = tree.range(key..=key);
        let entry = if forward {
            scan.next()
        } else {
            scan.next_back()
        };
        assert_eq!(entry.unwrap().unwrap().into_pair().unwrap(), (key, key));
        assert!(scan.next().is_none());
        assert!(scan.next_back().is_none());
        // Deferring the read must not hide corruption when traversal continues.
        let mut continuing = if forward {
            tree.range(key..)
        } else {
            tree.range(..=key)
        };
        let entry = if forward {
            continuing.next()
        } else {
            continuing.next_back()
        };
        assert_eq!(entry.unwrap().unwrap().into_pair().unwrap(), (key, key));
        let next = if forward {
            continuing.next()
        } else {
            continuing.next_back()
        };
        assert!(matches!(next, Some(Err(Error::Checksum))));
        assert!(continuing.next().is_none());
        assert!(continuing.next_back().is_none());
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn splitting_a_corrupt_free_block_refuses_without_publishing_any_halves() {
    let backing = VectorMemory::default();
    let mut tree = Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    tree.insert(1, vec![7; 4096]).unwrap();
    let Value::External(p) = tree.find(&1).unwrap().unwrap() else {
        panic!()
    };
    tree.insert(1, vec![1]).unwrap();
    backing.borrow_mut()[usize::try_from(p.offset).unwrap() + 12] ^= 1;
    let before = backing.borrow().clone();
    assert!(matches!(tree.insert(2, vec![2; 80]), Err(Error::Checksum)));
    assert_eq!(*backing.borrow(), before);
    assert_eq!(tree.get(&1).unwrap(), Some(vec![1]));
    assert_eq!(tree.get(&2).unwrap(), None);
}

#[test]
fn large_free_blocks_split_without_losing_capacity_or_overlapping_live_records() {
    let backing = VectorMemory::default();
    let mut tree = Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    tree.insert(1, vec![7; 4096]).unwrap();
    let Value::External(original) = tree.find(&1).unwrap().unwrap() else {
        panic!()
    };
    tree.insert(1, vec![1]).unwrap();
    let cursor = tree.root.borrow().cursor;
    tree.insert(2, vec![2; 80]).unwrap();
    let Value::External(reused) = tree.find(&2).unwrap().unwrap() else {
        panic!()
    };
    assert_eq!(reused.offset, original.offset);
    assert_eq!(reused.capacity(), 256);
    assert_eq!(tree.root.borrow().cursor, cursor);
    drop(tree);
    let mut tree = Tree::<u64, Vec<u8>, _>::load(backing).unwrap();
    assert_eq!(tree.get(&1).unwrap(), Some(vec![1]));
    assert_eq!(tree.get(&2).unwrap(), Some(vec![2; 80]));

    // Every byte of the original block is either the reused live value or one
    // nonoverlapping free half. Check persisted links after reopening.
    let mut ranges = vec![(reused.offset, reused.offset + reused.capacity())];
    for mut head in tree.root.borrow().free.clone() {
        let mut seen = BTreeSet::new();
        while let Some(p) = head {
            assert!(seen.insert(p.offset));
            if p.offset >= original.offset && p.offset < original.offset + original.capacity() {
                ranges.push((p.offset, p.offset + p.capacity()));
            }
            let bytes = record::read_bytes(&tree.memory, p.offset, p.capacity(), FREE).unwrap();
            head = record::decode(FREE, &bytes).unwrap();
        }
    }
    ranges.sort_unstable();
    assert_eq!(ranges.first().unwrap().0, original.offset);
    assert_eq!(
        ranges.last().unwrap().1,
        original.offset + original.capacity()
    );
    assert!(ranges.windows(2).all(|w| w[0].1 == w[1].0));
    for key in 3..24 {
        tree.insert(key, vec![u8::try_from(key).unwrap(); 80])
            .unwrap();
    }
    check(&tree);
    assert_eq!(tree.get(&2).unwrap(), Some(vec![2; 80]));
}

// Check persisted shape independently of lookup results: equal leaf depth,
// occupancy, separator ordering and unique live routing/value blocks.
fn check(tree: &Tree<u64, Vec<u8>, VectorMemory>) {
    fn walk(
        tree: &Tree<u64, Vec<u8>, VectorMemory>,
        p: Pointer,
        root: bool,
        low: Option<u64>,
        high: Option<u64>,
        live: &mut BTreeSet<(u64, u64)>,
    ) -> (u64, usize) {
        assert!(live.insert((p.offset, p.offset + p.capacity())));
        let node = tree.node(p).unwrap();
        assert!(!node.entries.is_empty());
        assert!(root || node.entries.len() >= MIN_ENTRIES);
        assert!(node.entries.len() <= MAX_ENTRIES);
        let keys: Vec<_> = node
            .entries
            .iter()
            .map(|e| decode::<u64>(e.key.clone()).unwrap())
            .collect();
        assert!(keys.windows(2).all(|w| w[0] < w[1]));
        assert!(low.is_none_or(|k| keys[0] > k));
        assert!(high.is_none_or(|k| *keys.last().unwrap() < k));
        for entry in &node.entries {
            if let Value::External(p) = entry.value {
                assert!(live.insert((p.offset, p.offset + p.capacity())));
            }
        }
        let mut count = keys.len() as u64;
        let mut depth = None;
        for (i, p) in node.children.iter().enumerate() {
            let (n, d) = walk(
                tree,
                *p,
                false,
                if i == 0 { low } else { Some(keys[i - 1]) },
                keys.get(i).copied().or(high),
                live,
            );
            count += n;
            assert!(depth.is_none_or(|expected| expected == d));
            depth = Some(d);
        }
        (count, depth.map_or(1, |d| d + 1))
    }
    let mut ranges = BTreeSet::new();
    if let Some(top) = tree.root.borrow().top {
        assert_eq!(walk(tree, top, true, None, None, &mut ranges).0, tree.len());
    } else {
        assert_eq!(tree.len(), 0);
    }
    for (order, mut head) in tree.root.borrow().free.iter().copied().enumerate() {
        while let Some(pointer) = head {
            assert_eq!(usize::from(pointer.order), order);
            assert!(ranges.insert((pointer.offset, pointer.offset + pointer.capacity())));
            let bytes =
                record::read_bytes(&tree.memory, pointer.offset, pointer.capacity(), FREE).unwrap();
            head = record::decode(FREE, &bytes).unwrap();
        }
    }
    let mut end = record::HEADER_BYTES;
    for (start, next) in ranges {
        assert!(start >= end, "overlapping live/free blocks at {start}");
        end = next;
    }
}

#[test]
fn persistent_splits_rotations_merges_and_root_collapse_preserve_btree_invariants() {
    let backing = VectorMemory::default();
    let mut tree = Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
    for key in (0..2400).map(|i| (i * 997) % 2400) {
        tree.insert(key, vec![u8::try_from(key % 256).unwrap(); 80])
            .unwrap();
        if key % 97 == 0 {
            check(&tree);
        }
    }
    check(&tree);
    for key in (0..2400).map(|i| (i * 991) % 2400) {
        assert!(tree.remove(&key).unwrap().is_some());
        if key % 97 == 0 {
            drop(tree);
            tree = Tree::load(backing.clone()).unwrap();
            check(&tree);
        }
    }
    check(&tree);
}

#[test]
fn malformed_routing_and_value_records_refuse_without_writes() {
    for mutation in 0..5 {
        let backing = VectorMemory::default();
        let mut tree = Tree::<u64, Vec<u8>, _>::reset(backing.clone()).unwrap();
        tree.insert(1, vec![7; 1024]).unwrap();
        let p = tree.root.borrow().top.unwrap();
        let node = tree.node(p).unwrap();
        if mutation == 4 {
            let Value::External(v) = node.entries[0].value else {
                panic!()
            };
            backing.borrow_mut()[usize::try_from(v.offset).unwrap() + 20] ^= 1;
        } else {
            let bytes = record::read_bytes(&backing, p.offset, p.capacity(), NODE).unwrap();
            let mut payload = record::decode_payload(NODE, &bytes).unwrap().to_vec();
            match mutation {
                0 => payload[0] = 255,
                1 => payload[1] = 255,
                2 => payload[2..6].copy_from_slice(&u32::MAX.to_le_bytes()),
                3 => {
                    payload.pop();
                }
                _ => unreachable!(),
            }
            backing.write(p.offset, &record::encode_payload(NODE, &payload).unwrap());
        }
        let before = backing.borrow().clone();
        assert!(tree.get(&1).is_err());
        assert!(tree.insert(1, vec![9; 1024]).is_err());
        assert!(tree.remove(&1).is_err());
        assert_eq!(*backing.borrow(), before);
    }
}
