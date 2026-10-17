//! Differential qualification against DFINITY 0.7.2 and an independent heap model.
use super::super::BTreeMap;
use super::super::{Comparator, Config, Error};
use crate::substrate as ic_stable_structures;
use ic_stable_structures::{BTreeMap as Upstream, Storable, VectorMemory, storable};
use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap as Model, ops::Bound};

// Ord uses decoded bytes; the wire length prefix deliberately changes byte order.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Key(Vec<u8>);
impl Storable for Key {
    const BOUND: storable::Bound = storable::Bound::Unbounded;
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut bytes = u32::try_from(self.0.len()).unwrap().to_be_bytes().to_vec();
        bytes.extend_from_slice(&self.0);
        Cow::Owned(bytes)
    }
    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(decode(&bytes).unwrap().to_vec())
    }
}
fn decode(bytes: &[u8]) -> Result<&[u8], Error> {
    let prefix = bytes.get(..4).ok_or(Error::InvalidKey)?;
    let len = u32::from_be_bytes(prefix.try_into().map_err(|_| Error::InvalidKey)?);
    let payload = &bytes[4..];
    if payload.len() != usize::try_from(len).map_err(|_| Error::InvalidKey)? {
        return Err(Error::InvalidKey);
    }
    Ok(payload)
}
struct Semantic;
impl Comparator for Semantic {
    const IDENTITY: [u8; 16] = *b"upstream-test001";
    fn compare(&self, a: &[u8], b: &[u8]) -> Result<Ordering, Error> {
        Ok(decode(a)?.cmp(decode(b)?))
    }
}
struct Harness {
    ours: BTreeMap<VectorMemory, Semantic>,
    upstream: Upstream<Key, Vec<u8>, VectorMemory>,
    model: Model<Vec<u8>, Vec<u8>>,
}
impl Harness {
    fn new() -> Self {
        Self {
            ours: BTreeMap::create(VectorMemory::default(), Config::default(), Semantic).unwrap(),
            upstream: Upstream::new(VectorMemory::default()),
            model: Model::new(),
        }
    }
    fn insert(&mut self, key: &Key, value: Vec<u8>) {
        let expected = self.model.insert(key.0.clone(), value.clone());
        assert_eq!(self.ours.insert(&key.to_bytes(), &value).unwrap(), expected);
        assert_eq!(self.upstream.insert(key.clone(), value), expected);
    }
    fn remove(&mut self, key: &Key) {
        let expected = self.model.remove(&key.0);
        assert_eq!(self.ours.remove(&key.to_bytes()).unwrap(), expected);
        assert_eq!(self.upstream.remove(key), expected);
    }
    fn check(&self) {
        self.ours.validate().unwrap();
        assert_eq!(self.ours.len(), self.model.len() as u64);
        assert_eq!(self.upstream.len(), self.ours.len());
        let ours = self
            .ours
            .iter()
            .map(|entry| {
                let (key, value) = entry.unwrap().into_pair().unwrap();
                (decode(&key).unwrap().to_vec(), value)
            })
            .collect::<Vec<_>>();
        let upstream = self
            .upstream
            .iter()
            .map(|entry| {
                let (key, value) = entry.into_pair();
                (key.0, value)
            })
            .collect::<Vec<_>>();
        assert_eq!(ours, self.model.clone().into_iter().collect::<Vec<_>>());
        assert_eq!(upstream, ours);
    }
    fn reopen(self) -> Self {
        Self {
            ours: BTreeMap::open(self.ours.into_memory(), Config::default(), Semantic).unwrap(),
            upstream: Upstream::load(self.upstream.into_memory()),
            model: self.model,
        }
    }
    fn range(&self, lower: Bound<Key>, upper: Bound<Key>, mut random: u64) {
        let encoded_lower = lower.as_ref().map(|key| key.to_bytes().into_owned());
        let encoded_upper = upper.as_ref().map(|key| key.to_bytes().into_owned());
        let mut ours = self
            .ours
            .range(
                encoded_lower.as_ref().map(Vec::as_slice),
                encoded_upper.as_ref().map(Vec::as_slice),
            )
            .unwrap();
        let mut upstream = self.upstream.range((lower.clone(), upper.clone()));
        let mut expected = self
            .model
            .range((lower.map(|key| key.0), upper.map(|key| key.0)));
        loop {
            let reverse = next_random(&mut random) & 1 != 0;
            let item = if reverse {
                expected.next_back()
            } else {
                expected.next()
            };
            let own = if reverse {
                ours.next_back()
            } else {
                ours.next()
            };
            let other = if reverse {
                upstream.next_back()
            } else {
                upstream.next()
            };
            let Some((key, value)) = item else {
                assert!(own.is_none());
                assert!(other.is_none());
                break;
            };
            let own = own.unwrap().unwrap();
            let other = other.unwrap();
            assert_eq!(decode(own.key()).unwrap(), key);
            assert_eq!(&other.key().0, key);
            assert_eq!(&own.value().unwrap(), value);
            assert_eq!(&other.value(), value);
        }
        assert!(ours.next().is_none() && ours.next_back().is_none());
        assert!(upstream.next().is_none() && upstream.next_back().is_none());
    }
}
fn next_random(state: &mut u64) -> u64 {
    // High bits select operations; avoid coupling LCG low bits to action cycles.
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    *state >> 16
}
fn run_seed(seed: u64, steps: usize) {
    let mut state = seed;
    let keys = (0..768)
        .map(|n| Key(format!("{n}{}", "x".repeat(n % 80)).into_bytes()))
        .collect::<Vec<_>>();
    let mut harness = Harness::new();
    for key in &keys {
        harness.insert(key, vec![0]);
    }
    for step in 0..steps {
        let random = next_random(&mut state);
        let key = &keys[usize::try_from(random % keys.len() as u64).unwrap()];
        match (random >> 12) % 5 {
            0 => harness.remove(key),
            1 => {
                let expected = harness.model.get(&key.0).cloned();
                assert_eq!(harness.ours.get(&key.to_bytes()).unwrap(), expected);
                assert_eq!(harness.upstream.get(key), expected);
                assert_eq!(
                    harness.ours.contains_key(&key.to_bytes()).unwrap(),
                    expected.is_some()
                );
                assert_eq!(harness.upstream.contains_key(key), expected.is_some());
            }
            _ => {
                let size =
                    [0, 1, 16, 17, 4072, 4073, 8192][usize::try_from((random >> 24) % 7).unwrap()];
                harness.insert(key, vec![u8::try_from(random % 251).unwrap(); size]);
            }
        }
        if step % 97 == 0 {
            let a = keys[usize::try_from(next_random(&mut state) % 768).unwrap()].clone();
            let b = keys[usize::try_from(next_random(&mut state) % 768).unwrap()].clone();
            let (a, b) = if a <= b { (a, b) } else { (b, a) };
            harness.range(
                Bound::Included(a.clone()),
                Bound::Excluded(b.clone()),
                random,
            );
            harness.range(Bound::Excluded(a), Bound::Included(b), !random);
            harness.range(Bound::Unbounded, Bound::Unbounded, random);
        }
        if step % 257 == 0 {
            harness.check();
            harness = harness.reopen();
            harness.check();
        }
    }
    harness.check();
    // Delete all entries in a permutation to force merge/root-shrink paths.
    for n in 0..keys.len() {
        harness.remove(&keys[(n * 109) % keys.len()]);
    }
    harness = harness.reopen();
    harness.check();
    assert!(harness.ours.is_empty() && harness.upstream.is_empty());
    // Refill after draining to exercise free-page reuse in each implementation.
    for key in keys.iter().rev() {
        harness.insert(key, vec![9; 17]);
    }
    harness.reopen().check();
}
#[test]
fn seeded_operations_match_upstream_and_model() {
    for seed in [1, 17, 42, 0xdead_beef] {
        eprintln!("differential seed={seed}");
        run_seed(seed, 2000);
    }
}
#[test]
#[ignore = "extended differential campaign; run explicitly"]
fn extended_seeded_operations_match_upstream_and_model() {
    for seed in 0..32 {
        eprintln!("extended differential seed={seed}");
        run_seed(seed, 10_000);
    }
}
#[test]
fn maximum_payloads_and_boundary_ranges_match_upstream() {
    let mut harness = Harness::new();
    for len in [0, 1, 28, 29, 4072, 128 * 1024 - 4] {
        let key = Key(vec![7; len]);
        harness.insert(&key, vec![8; 4 * 1024 * 1024]);
        harness = harness.reopen();
        harness.check();
        harness.range(
            Bound::Included(key.clone()),
            Bound::Included(key.clone()),
            1,
        );
        harness.range(
            Bound::Excluded(key.clone()),
            Bound::Included(key.clone()),
            0,
        );
        harness.insert(&key, vec![]);
        harness.insert(&key, vec![1; 65536]);
        harness.remove(&key);
        harness.check();
    }
}

#[test]
fn all_valid_bound_pairs_match_upstream() {
    let mut harness = Harness::new();
    let keys = (0..80)
        .map(|n| Key(format!("{n}{}", "x".repeat(n % 35)).into_bytes()))
        .collect::<Vec<_>>();
    for key in &keys {
        harness.insert(key, vec![3; 17]);
    }
    let mut endpoints = vec![Key(vec![]), Key(b"missing".to_vec()), Key(vec![255])];
    endpoints.extend(keys.iter().step_by(7).cloned());
    let mut bounds = vec![Bound::Unbounded];
    for key in endpoints {
        bounds.push(Bound::Included(key.clone()));
        bounds.push(Bound::Excluded(key));
    }
    for lower in &bounds {
        for upper in &bounds {
            if let (
                Bound::Included(a) | Bound::Excluded(a),
                Bound::Included(b) | Bound::Excluded(b),
            ) = (lower, upper)
            {
                // These are outside std::BTreeMap's accepted range contract.
                if a > b
                    || (a == b
                        && matches!(lower, Bound::Excluded(_))
                        && matches!(upper, Bound::Excluded(_)))
                {
                    continue;
                }
            }
            harness.range(lower.clone(), upper.clone(), 17);
        }
    }
    harness.reopen().check();
}

#[test]
fn upstream_bytes_are_rejected_without_modification() {
    let mut upstream = Upstream::<Key, Vec<u8>, _>::new(VectorMemory::default());
    upstream.insert(Key(b"retained".to_vec()), vec![9; 65536]);
    let memory = upstream.into_memory();
    let before = memory.borrow().clone();
    assert!(matches!(
        BTreeMap::open(memory.clone(), Config::default(), Semantic),
        Err(Error::Format)
    ));
    assert!(matches!(
        BTreeMap::create(memory.clone(), Config::default(), Semantic),
        Err(Error::NotEmpty)
    ));
    assert_eq!(*memory.borrow(), before);
    let upstream = Upstream::<Key, Vec<u8>, _>::load(memory);
    assert_eq!(
        upstream.get(&Key(b"retained".to_vec())),
        Some(vec![9; 65536])
    );
}
