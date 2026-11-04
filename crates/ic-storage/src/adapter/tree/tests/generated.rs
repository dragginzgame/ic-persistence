//! Seeded state-machine histories with deletion shrinking on failure.
use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Clone, Debug)]
enum Op {
    Insert(u64, usize, u8),
    Remove(u64),
    Get(u64),
    Range(u64, u64),
    Reopen,
    Clear,
}

fn replay(ops: &[Op], budget: usize) {
    let memory = VectorMemory::default();
    let mut tree = Tree::<u64, Vec<u8>, _>::reset(memory.clone()).unwrap();
    tree.set_cache_budget(budget);
    let mut model = BTreeMap::new();
    for op in ops {
        match *op {
            Op::Insert(key, size, byte) => {
                let value = vec![byte; size];
                assert_eq!(
                    tree.insert(key, value.clone()).unwrap(),
                    model.insert(key, value)
                );
            }
            Op::Remove(key) => assert_eq!(tree.remove(&key).unwrap(), model.remove(&key)),
            Op::Get(key) => assert_eq!(tree.get(&key).unwrap(), model.get(&key).cloned()),
            Op::Range(a, b) => {
                let mut actual = tree.range(a..=b);
                let mut expected = model.range(a..=b);
                let mut forward = true;
                loop {
                    let (entry, reference) = if forward {
                        (actual.next(), expected.next())
                    } else {
                        (actual.next_back(), expected.next_back())
                    };
                    assert_eq!(
                        entry.map(|e| e.unwrap().into_pair().unwrap()),
                        reference.map(|(k, v)| (*k, v.clone()))
                    );
                    if reference.is_none() {
                        break;
                    }
                    forward = !forward;
                }
            }
            Op::Reopen => {
                drop(tree);
                tree = Tree::load(memory.clone()).unwrap();
                tree.set_cache_budget(budget);
            }
            Op::Clear => {
                tree.clear().unwrap();
                model.clear();
            }
        }
        assert_eq!(tree.len(), model.len() as u64);
        // Inspect retained bytes through a cold reader, independently of cache.
        check::<u64>(&Tree::load(memory.clone()).unwrap());
    }
    assert_eq!(
        tree.range(..)
            .map(|e| e.unwrap().into_pair().unwrap())
            .collect::<Vec<_>>(),
        model.into_iter().collect::<Vec<_>>()
    );
}

fn minimize(mut ops: Vec<Op>, budget: usize) -> Vec<Op> {
    let mut chunk = ops.len().div_ceil(2);
    while chunk != 0 {
        let mut start = 0;
        while start < ops.len() {
            let mut candidate = ops.clone();
            candidate.drain(start..(start + chunk).min(candidate.len()));
            if catch_unwind(AssertUnwindSafe(|| replay(&candidate, budget))).is_err() {
                ops = candidate;
            } else {
                start += chunk;
            }
        }
        chunk /= 2;
    }
    ops
}

#[test]
fn generated_histories_match_ordered_model_with_cache_collisions_and_reopen() {
    for seed in 1_u64..=16 {
        let mut random = seed;
        let mut ops = Vec::new();
        for step in 0..500 {
            random = random
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let key = (random >> 32) % 191;
            ops.push(match random % 16 {
                0 => Op::Reopen,
                1 if step % 13 == 0 => Op::Clear,
                1..=3 => Op::Remove(key),
                4 => Op::Get(key),
                5 => Op::Range(key / 2, key),
                _ => Op::Insert(
                    key,
                    [0, 31, 32, 33, 236, 237, 4076, 4077, 4096][(random >> 16) as usize % 9],
                    random.to_le_bytes()[0],
                ),
            });
        }
        for budget in [0, 4096, 128 * 1024] {
            assert!(
                catch_unwind(AssertUnwindSafe(|| replay(&ops, budget))).is_ok(),
                "seed={seed}, budget={budget}, minimized history={:?}",
                minimize(ops.clone(), budget)
            );
        }
    }
}

#[test]
fn variable_keys_and_size_class_churn_preserve_cold_tree_and_free_lists() {
    // Vary routing-record capacity as well as external values. The eight-byte
    // prefix preserves numeric ordering while suffix lengths change node sizes.
    let keys: Vec<Vec<u8>> = (0_u8..72)
        .map(|id| {
            let mut key = u64::from(id).to_be_bytes().to_vec();
            key.resize(8 + [0, 31, 237, 1024][usize::from(id) % 4], id);
            key
        })
        .collect();
    let sizes = [0, 31, 32, 33, 236, 237, 4076, 4077, 65516, 65517];
    for budget in [0, 4096, 128 * 1024] {
        let memory = VectorMemory::default();
        let mut tree = Tree::<Vec<u8>, Vec<u8>, _>::reset(memory.clone()).unwrap();
        tree.set_cache_budget(budget);
        let mut model = BTreeMap::new();
        for round in 0..12 {
            for (index, key) in keys.iter().enumerate() {
                if (index + round) % 5 == 0 {
                    assert_eq!(tree.remove(key).unwrap(), model.remove(key));
                } else {
                    let value =
                        vec![u8::try_from(round).unwrap(); sizes[(index + round) % sizes.len()]];
                    assert_eq!(
                        tree.insert(key.clone(), value.clone()).unwrap(),
                        model.insert(key.clone(), value)
                    );
                }
            }
            drop(tree);
            tree = Tree::load(memory.clone()).unwrap();
            // Independently inspect ordering, depth, occupancy and disjoint
            // live/free blocks before enabling the cache on the reopened tree.
            check(&tree);
            tree.set_cache_budget(budget);
            assert_eq!(
                tree.range(..)
                    .map(|e| e.unwrap().into_pair().unwrap())
                    .collect::<Vec<_>>(),
                model
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<_>>()
            );
            for key in &keys {
                assert_eq!(tree.get(key).unwrap(), model.get(key).cloned());
            }
        }
        for key in keys.iter().rev() {
            assert_eq!(tree.remove(key).unwrap(), model.remove(key));
        }
        check::<Vec<u8>>(&Tree::load(memory).unwrap());
        assert_eq!(tree.len(), 0);
    }
}
