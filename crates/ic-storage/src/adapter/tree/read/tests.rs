use super::*;

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
    let other = (4352..1_000_000)
        .step_by(256)
        .map(|offset| Pointer { offset, order: 8 })
        .find(|p| cache.index(*p) == cache.index(root))
        .unwrap();
    let node = Rc::new(
        ReadNode::decode::<u64, u64>(Node::default().encode().unwrap(), 1_000_000).unwrap(),
    );
    cache.put(root, Rc::clone(&node), 0);
    cache.put(other, Rc::clone(&node), 3);
    assert!(cache.get(root, 0).is_some());
    assert!(cache.get(other, 3).is_none());
    assert_eq!(cache.metrics.protected_rejections, 1);
    assert_eq!(cache.metrics.hits, 1);
    assert_eq!(cache.metrics.collision_misses, 1);
    cache.invalidate(root.offset);
    cache.put(other, node, 1);
    assert!(cache.get(other, 1).is_some());
    assert!(cache.used <= cache.limit);
    cache.clear();
    assert!(cache.get(other, 1).is_none());
    assert_eq!(cache.metrics.cold_misses, 1);
}

#[test]
fn tiny_cache_budgets_include_the_slot_table() {
    let mut cache = NodeCache::default();
    for budget in 0..2048 {
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
