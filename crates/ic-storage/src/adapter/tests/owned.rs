//! Public collection histories, independent of engine-private layouts.
use super::*;
#[test]
fn vector_model_mixed_updates_reopen_and_bidirectional_iteration() {
    let backing = VectorMemory::default();
    let mut expected = Vec::new();
    for generation in 0..6 {
        let runtime = runtime(backing.clone());
        let memory = runtime
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
    }
}

#[test]
fn generated_map_histories_match_an_independent_model_after_reopen() {
    use std::collections::BTreeMap;
    for seed in 1u64..=16 {
        let backing = VectorMemory::default();
        let runtime = runtime(backing);
        let key = crate::StableKey::parse("test.map.v1").unwrap();
        let mut map =
            StableBTreeMap::<u64, Vec<u8>>::create(runtime.open_memory(&key).unwrap()).unwrap();
        let mut model = BTreeMap::new();
        let mut random = seed;
        for step in 0..500 {
            random = random
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let id = (random >> 32) % 191;
            match random % 16 {
                0 => {
                    drop(map);
                    map = StableBTreeMap::load(runtime.open_memory(&key).unwrap()).unwrap();
                }
                1 if step % 13 == 0 => {
                    map.clear().unwrap();
                    model.clear();
                }
                1..=3 => assert_eq!(map.remove(&id).unwrap(), model.remove(&id)),
                4 => assert_eq!(map.get(&id).unwrap(), model.get(&id).cloned()),
                5 => {
                    let mut actual = map.range(id / 2..=id);
                    let mut expected = model.range(id / 2..=id);
                    let mut forward = true;
                    loop {
                        let (a, b) = if forward {
                            (actual.next(), expected.next())
                        } else {
                            (actual.next_back(), expected.next_back())
                        };
                        assert_eq!(
                            a.map(|e| e.unwrap().into_pair().unwrap()),
                            b.map(|(k, v)| (*k, v.clone()))
                        );
                        if b.is_none() {
                            break;
                        }
                        forward = !forward;
                    }
                }
                _ => {
                    let n = [0, 31, 32, 33, 236, 237, 4076, 4077, 4096]
                        [usize::try_from((random >> 16) % 9).unwrap()];
                    let bytes = vec![random.to_le_bytes()[0]; n];
                    assert_eq!(
                        map.insert(id, bytes.clone()).unwrap(),
                        model.insert(id, bytes)
                    );
                }
            }
            assert_eq!(map.len(), model.len() as u64);
            if step % 64 == 0 {
                assert_eq!(
                    map.iter()
                        .map(|e| e.unwrap().into_pair().unwrap())
                        .collect::<Vec<_>>(),
                    model
                        .iter()
                        .map(|(k, v)| (*k, v.clone()))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}
