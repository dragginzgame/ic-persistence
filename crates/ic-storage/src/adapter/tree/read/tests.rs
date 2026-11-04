use super::*;

#[test]
fn inline_replacement_matches_full_encoding_at_every_position_and_length() {
    let pointer = Pointer {
        offset: 4096,
        order: 8,
    };
    for branch in [false, true] {
        for old_len in 0..=INLINE {
            let mut node = Node {
                entries: (0..MAX_ENTRIES)
                    .map(|index| Entry {
                        key: vec![u8::try_from(index).unwrap(); index + 1],
                        value: Value::Inline(vec![7; old_len]),
                    })
                    .collect(),
                children: if branch {
                    vec![pointer; MAX_ENTRIES + 1]
                } else {
                    vec![]
                },
            };
            // Unchanged external descriptors must also survive byte-for-byte.
            node.entries[1].value = Value::External(pointer);
            let read = ReadNode::decode::<Vec<u8>, Vec<u8>>(node.encode().unwrap(), 8192).unwrap();
            for index in (0..MAX_ENTRIES).filter(|index| *index != 1) {
                for new_len in 0..=INLINE {
                    let value = vec![9; new_len];
                    let mut expected = node.clone();
                    expected.entries[index].value = Value::Inline(value.clone());
                    assert_eq!(
                        read.replace_inline(index, &value).unwrap(),
                        expected.encode().unwrap()
                    );
                }
            }
            assert!(read.replace_inline(1, &[]).is_err());
            assert!(read.replace_inline(0, &[0; INLINE + 1]).is_err());
        }
    }
}

#[test]
fn capacity_scales_and_routing_nodes_survive_leaf_collisions() {
    let mut cache = NodeCache::default();
    cache.configure(32 * 1024);
    let small = cache.slots.len();
    cache.configure(128 * 1024);
    assert!(cache.slots.len() > small && cache.slots.len() > 32);
    let root = Pointer {
        offset: 4096,
        order: 8,
    };
    let colliders: Vec<_> = (4352..1_000_000)
        .step_by(256)
        .map(|offset| Pointer { offset, order: 8 })
        .filter(|p| cache.index(*p) == cache.index(root))
        .take(3)
        .collect();
    let node = Rc::new(
        ReadNode::decode::<u64, u64>(Node::default().encode().unwrap(), 1_000_000).unwrap(),
    );
    cache.put(root, Rc::clone(&node), 0);
    cache.put(colliders[0], Rc::clone(&node), 3);
    // A protected ancestor and a colliding leaf can now both stay warm.
    for _ in 0..32 {
        assert!(cache.get(root, 0).is_some());
        assert!(cache.get(colliders[0], 3).is_some());
    }
    assert_eq!(cache.metrics.protected_rejections, 0);
    cache.put(colliders[1], Rc::clone(&node), 1);
    assert!(cache.get(root, 0).is_some());
    assert!(cache.get(colliders[1], 1).is_some());
    cache.put(colliders[2], Rc::clone(&node), 3);
    assert!(cache.get(colliders[2], 3).is_none());
    assert_eq!(cache.metrics.protected_rejections, 1);
    // Invalidate either way, then reuse the free way without evicting its neighbour.
    cache.invalidate(root.offset);
    cache.put(colliders[2], Rc::clone(&node), 3);
    assert!(cache.get(colliders[2], 3).is_some());
    assert!(cache.get(colliders[1], 1).is_some());
    cache.invalidate(colliders[2].offset);
    assert!(cache.get(colliders[2], 3).is_none());
    assert!(cache.used <= cache.limit);
    cache.clear();
    assert!(cache.get(colliders[1], 1).is_none());
    assert_eq!(
        cache.used,
        cache.slots.capacity() * size_of::<Option<Cached>>()
    );
}

#[test]
fn colliding_leaves_evict_the_least_recent_and_keep_exact_accounting() {
    let mut cache = NodeCache::default();
    cache.configure(4096); // One set on both native and Wasm.
    let pointers = [4096, 4352, 4608].map(|offset| Pointer { offset, order: 8 });
    let node =
        Rc::new(ReadNode::decode::<u64, u64>(Node::default().encode().unwrap(), 8192).unwrap());
    cache.put(pointers[0], Rc::clone(&node), 3);
    cache.put(pointers[1], Rc::clone(&node), 3);
    assert!(cache.get(pointers[0], 3).is_some());
    cache.put(pointers[2], Rc::clone(&node), 3);
    assert!(cache.get(pointers[0], 3).is_some());
    assert!(cache.get(pointers[1], 3).is_none());
    assert!(cache.get(pointers[2], 3).is_some());
    let used = cache.used;
    cache.put(pointers[2], Rc::clone(&node), 3);
    assert_eq!(cache.used, used);
    cache.limit = used;
    let large = Node {
        entries: vec![Entry {
            key: 1_u64.to_le_bytes().to_vec(),
            value: Value::Inline(vec![1; 8]),
        }],
        children: vec![],
    };
    let large = Rc::new(ReadNode::decode::<u64, u64>(large.encode().unwrap(), 8192).unwrap());
    cache.put(pointers[1], large, 3);
    assert_eq!(cache.metrics.budget_rejections, 1);
    assert_eq!(cache.used, used);
    assert!(cache.get(pointers[0], 3).is_some());
    assert!(cache.get(pointers[2], 3).is_some());
}

#[test]
fn tiny_cache_budgets_include_the_slot_table() {
    let mut cache = NodeCache::default();
    for budget in 0..4096 {
        cache.configure(budget);
        assert!(cache.used <= budget, "budget={budget}, used={}", cache.used);
        let node =
            Rc::new(ReadNode::decode::<u64, u64>(Node::default().encode().unwrap(), 4096).unwrap());
        cache.put(
            Pointer {
                offset: 4096,
                order: 8,
            },
            node,
            0,
        );
        assert!(cache.used <= budget);
        cache.clear();
        assert!(cache.used <= budget);
    }
}
