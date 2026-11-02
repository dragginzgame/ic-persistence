use super::*;
use crate::VectorMemory;
use std::collections::BTreeSet;

// Check persisted shape independently of lookup results: equal leaf depth,
// occupancy, separator ordering and unique live routing/value blocks.
fn check(tree: &Tree<u64, Vec<u8>, VectorMemory>) {
    fn walk(
        tree: &Tree<u64, Vec<u8>, VectorMemory>,
        p: Pointer,
        root: bool,
        low: Option<u64>,
        high: Option<u64>,
        live: &mut BTreeSet<u64>,
    ) -> (u64, usize) {
        assert!(live.insert(p.offset));
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
                assert!(live.insert(p.offset));
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
    if let Some(top) = tree.root.borrow().top {
        assert_eq!(
            walk(tree, top, true, None, None, &mut BTreeSet::new()).0,
            tree.len()
        );
    } else {
        assert_eq!(tree.len(), 0);
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
