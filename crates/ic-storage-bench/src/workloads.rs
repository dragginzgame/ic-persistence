//! Source-informed canonical-store workloads; not an `IcyDB` codec or overlay.
use super::{Bytes, Memory, Tree, Upstream};
use ic_storage::{BTreeMap, Config, Lexicographic};
use ic_testkit::performance::Performance;
use std::{cell::RefCell, collections::BTreeMap as Model, ops::Bound};

struct State {
    tree: Tree,
    model: Model<Vec<u8>, Vec<u8>>,
    index: bool,
    seeded: u64,
    step: u8,
}
thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }

pub fn init(engine: u8, profile: usize) {
    assert!(engine <= 1 && profile < 2);
    let index = profile == 1;
    let memory = Memory::default();
    let tree = match (engine, index) {
        (0, _) => Tree::Ours(BTreeMap::create(memory, Config::default(), Lexicographic).unwrap()),
        (_, true) => Tree::Index(Upstream::new(memory)),
        (_, false) => Tree::Row(Upstream::new(memory)),
    };
    STATE.with_borrow_mut(|state| {
        *state = Some(State {
            tree,
            model: Model::new(),
            index,
            seeded: 0,
            step: 0,
        });
    });
}

fn key(n: u64) -> Vec<u8> {
    let len = [24, 32, 33, 64, 128, 262][usize::try_from(n % 6).unwrap()];
    let mut bytes = vec![0xa5; len];
    bytes[..8].copy_from_slice(&n.to_be_bytes());
    bytes
}

fn value(n: u64, generation: u8, index: bool) -> Vec<u8> {
    let len = if index {
        1
    } else {
        [1, 16, 17, 32, 256, 1024, 4072, 4073, 8192, 65536]
            [usize::try_from((n + u64::from(generation)) % 10).unwrap()]
    };
    (0..len)
        .map(|i| u8::try_from((i + usize::from(generation)) % 251).unwrap() + 1)
        .collect()
}

#[ic_cdk::update]
fn workload_seed() -> bool {
    STATE.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        let end = (state.seeded + 16).min(512);
        for i in state.seeded..end {
            // A full permutation gives a reproducible nonsequential insertion order.
            let n = (i * 149 + 37) % 512;
            let key = key(n);
            let value = value(n, 0, state.index);
            assert_eq!(state.tree.insert(key.clone(), value.clone()), None);
            assert_eq!(state.model.insert(key, value), None);
        }
        state.seeded = end;
        end == 512
    })
}

// Both engines copy just the requested projection into the output. Upstream's
// key-only cursor never calls value(); neither path clones a key into a Vec first.
fn scan(tree: &Tree, lower: Vec<u8>, upper: Vec<u8>, reverse: bool, values: bool) -> Vec<u8> {
    let mut output = Vec::new();
    match tree {
        Tree::Ours(tree) => {
            let mut iter = tree
                .range(Bound::Included(&lower), Bound::Excluded(&upper))
                .unwrap();
            for _ in 0..16 {
                let Some(entry) = (if reverse {
                    iter.next_back()
                } else {
                    iter.next()
                }) else {
                    break;
                };
                let entry = entry.unwrap();
                if values {
                    output.extend_from_slice(&entry.value().unwrap());
                } else {
                    output.extend_from_slice(entry.key());
                }
            }
        }
        Tree::Index(tree) => {
            let mut iter = tree.range(Bytes(lower)..Bytes(upper));
            for _ in 0..16 {
                let Some(entry) = (if reverse {
                    iter.next_back()
                } else {
                    iter.next()
                }) else {
                    break;
                };
                if values {
                    output.extend_from_slice(&entry.value().0);
                } else {
                    output.extend_from_slice(&entry.key().0);
                }
            }
        }
        Tree::Row(tree) => {
            let mut iter = tree.range(Bytes(lower)..Bytes(upper));
            for _ in 0..16 {
                let Some(entry) = (if reverse {
                    iter.next_back()
                } else {
                    iter.next()
                }) else {
                    break;
                };
                if values {
                    output.extend_from_slice(&entry.value().0);
                } else {
                    output.extend_from_slice(&entry.key().0);
                }
            }
        }
        Tree::Journal(_) => unreachable!(),
    }
    output
}

#[ic_cdk::update]
fn workload_measure(operation: u8) -> (u128, u128, u128, u64) {
    STATE.with_borrow_mut(|state| {
        let state = state.as_mut().unwrap();
        assert_eq!(state.seeded, 512);
        assert_eq!(operation, state.step % 8);
        assert!(state.step < 32);
        let round = state.step / 8;
        let lower = key(128 + u64::from(round));
        let upper = key(384 + u64::from(round));
        let reverse = operation == 5 || operation == 7;
        let values = operation >= 6;
        // Allocate inputs and calculate independent expected results before timing.
        let inputs = (0..32)
            .map(|i| {
                let n = (i * 149 + u64::from(round) * 43) % 512;
                let n = if operation == 0 && i % 4 == 0 {
                    n + 512
                } else {
                    n
                };
                (key(n), value(n, round + 1, state.index))
            })
            .collect::<Vec<_>>();
        let expected = if operation < 4 {
            inputs
                .iter()
                .map(|(key, _)| state.model.get(key).cloned())
                .collect::<Vec<_>>()
        } else {
            let mut iter = state.model.range(lower.clone()..upper.clone());
            let mut output = Vec::new();
            for _ in 0..16 {
                let Some((key, value)) = (if reverse {
                    iter.next_back()
                } else {
                    iter.next()
                }) else {
                    break;
                };
                output.extend_from_slice(if values { value } else { key });
            }
            vec![Some(output)]
        };
        let before = Performance::counters();
        let result = if operation < 4 {
            inputs
                .into_iter()
                .map(|(key, value)| match operation {
                    0 => state.tree.get(key),
                    1 | 3 => state.tree.insert(key, value),
                    2 => state.tree.remove(key),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
        } else {
            vec![Some(scan(&state.tree, lower, upper, reverse, values))]
        };
        let after = Performance::counters();
        assert_eq!(result, expected);
        for i in 0..32 {
            let n = (i * 149 + u64::from(round) * 43) % 512;
            match operation {
                1 | 3 => {
                    state.model.insert(key(n), value(n, round + 1, state.index));
                }
                2 => {
                    state.model.remove(&key(n));
                }
                _ => {}
            }
        }
        if matches!(operation, 1..=3) {
            // Check all rows after every mutation batch, outside the measured window.
            for n in 0..512 {
                assert_eq!(state.tree.get(key(n)), state.model.get(&key(n)).cloned());
            }
            if let Tree::Ours(tree) = &state.tree {
                tree.validate().unwrap();
            }
        }
        state.step += 1;
        (
            after.instructions - before.instructions,
            after.heap_bytes,
            after.memory_bytes,
            result
                .iter()
                .flatten()
                .map(|v| u64::try_from(v.len()).unwrap())
                .sum(),
        )
    })
}
