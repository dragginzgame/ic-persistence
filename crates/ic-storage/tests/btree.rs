//! Consumer-level differential, persistence, range and failure tests.
use ic_stable_structures::VectorMemory;
use ic_storage::{BTreeMap, Comparator, Config, Error, Lexicographic, Memory};
use std::{cell::Cell, cmp::Ordering, collections::BTreeMap as Model, ops::Bound, rc::Rc};

type Tree = BTreeMap<VectorMemory>;
fn fresh() -> Tree {
    Tree::create(VectorMemory::default(), Config::default(), Lexicographic).unwrap()
}
fn key(n: u64) -> Vec<u8> {
    n.to_be_bytes().to_vec()
}
fn pairs(tree: &Tree) -> Vec<(Vec<u8>, Vec<u8>)> {
    tree.iter()
        .map(|entry| entry.unwrap().into_pair().unwrap())
        .collect()
}
fn assert_model(tree: &Tree, model: &Model<Vec<u8>, Vec<u8>>) {
    tree.validate().unwrap();
    assert_eq!(tree.len(), model.len() as u64);
    assert_eq!(
        pairs(tree),
        model
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        tree.iter()
            .rev()
            .map(|e| e.unwrap().key().to_vec())
            .collect::<Vec<_>>(),
        model.keys().rev().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn split_merge_replace_reopen_and_reuse_pages() {
    let mut tree = fresh();
    let mut model = Model::new();
    for n in 0..2100 {
        let k = key(n);
        let v = vec![u8::try_from(n % 251).unwrap(); 50];
        assert_eq!(tree.insert(&k, &v).unwrap(), model.insert(k, v));
    }
    assert_model(&tree, &model);
    tree = Tree::open(tree.into_memory(), Config::default(), Lexicographic).unwrap();
    for n in (0..2100).rev() {
        let k = key(n);
        assert_eq!(
            tree.insert(&k, b"x").unwrap(),
            model.insert(k, b"x".to_vec())
        );
    }
    let capacity = tree.memory_bytes();
    // A coprime permutation forces internal deletion, borrowing and merging.
    for n in 0..2100 {
        let k = key((n * 109) % 2100);
        assert_eq!(tree.remove(&k).unwrap(), model.remove(&k));
        if n % 137 == 0 {
            assert_model(&tree, &model);
        }
    }
    assert!(tree.is_empty());
    assert_model(&tree, &model);
    tree = Tree::open(tree.into_memory(), Config::default(), Lexicographic).unwrap();
    for n in (0..2100).rev() {
        tree.insert(&key(n), &[7; 50]).unwrap();
    }
    tree.validate().unwrap();
    assert!(tree.memory_bytes() <= capacity);
}

#[test]
fn deterministic_mutations_match_standard_map() {
    let mut tree = fresh();
    let mut model = Model::new();
    let mut random = 17u64;
    for step in 0..12000 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let k = key((random >> 24) % 1200);
        match random % 4 {
            0 => assert_eq!(tree.remove(&k).unwrap(), model.remove(&k)),
            1 => assert_eq!(tree.get(&k).unwrap(), model.get(&k).cloned()),
            _ => {
                let v = vec![
                    u8::try_from(random % 255).unwrap();
                    usize::try_from((random >> 10) % 90).unwrap()
                ];
                assert_eq!(tree.insert(&k, &v).unwrap(), model.insert(k, v));
            }
        }
        if step % 401 == 0 {
            assert_model(&tree, &model);
            tree = Tree::open(tree.into_memory(), Config::default(), Lexicographic).unwrap();
        }
    }
    assert_model(&tree, &model);
}

#[test]
fn all_range_bounds_and_interleaved_directions() {
    let mut tree = fresh();
    let mut model = Model::new();
    for n in (0..1000).step_by(3) {
        tree.insert(&key(n), &key(n)).unwrap();
        model.insert(key(n), key(n));
    }
    let mut bounds = vec![Bound::Unbounded];
    for n in [0, 1, 15, 48, 49, 127, 300, 999, 1000] {
        bounds.push(Bound::Included(key(n)));
        bounds.push(Bound::Excluded(key(n)));
    }
    for lower in &bounds {
        for upper in &bounds {
            let within = |k: &&Vec<u8>| {
                let low = match lower {
                    Bound::Unbounded => true,
                    Bound::Included(v) => *k >= v,
                    Bound::Excluded(v) => *k > v,
                };
                let high = match upper {
                    Bound::Unbounded => true,
                    Bound::Included(v) => *k <= v,
                    Bound::Excluded(v) => *k < v,
                };
                low && high
            };
            let expected = model.keys().filter(within).cloned().collect::<Vec<_>>();
            let range = tree.range(
                lower.as_ref().map(Vec::as_slice),
                upper.as_ref().map(Vec::as_slice),
            );
            if let (
                Bound::Included(a) | Bound::Excluded(a),
                Bound::Included(b) | Bound::Excluded(b),
            ) = (lower, upper)
            {
                if a > b {
                    assert!(matches!(range, Err(Error::InvalidRange)));
                    continue;
                }
            }
            let mut range = range.unwrap();
            let (mut front, mut back) = (0, expected.len());
            while front < back {
                assert_eq!(range.next().unwrap().unwrap().key(), expected[front]);
                front += 1;
                if front < back {
                    back -= 1;
                    assert_eq!(range.next_back().unwrap().unwrap().key(), expected[back]);
                }
            }
            assert!(range.next().is_none());
            assert!(range.next_back().is_none());
        }
    }
}

struct Framed;
impl Comparator for Framed {
    const IDENTITY: [u8; 16] = *b"test-framed-key1";
    fn compare(&self, left: &[u8], right: &[u8]) -> Result<Ordering, Error> {
        let decode = |bytes: &[u8]| {
            if bytes.first().map(|n| usize::from(*n) + 1) != Some(bytes.len()) {
                return Err(Error::InvalidKey);
            }
            Ok(bytes[1..].to_vec())
        };
        Ok(decode(left)?.cmp(&decode(right)?))
    }
}
#[test]
fn semantic_key_order_differs_from_wire_order() {
    let mut tree = BTreeMap::create(VectorMemory::default(), Config::default(), Framed).unwrap();
    for k in [b"\x01z".as_slice(), b"\x02aa", b"\x01b"] {
        tree.insert(k, b"present").unwrap();
    }
    assert_eq!(
        tree.iter()
            .map(|e| e.unwrap().key().to_vec())
            .collect::<Vec<_>>(),
        vec![b"\x02aa".to_vec(), b"\x01b".to_vec(), b"\x01z".to_vec()]
    );
    assert_eq!(tree.insert(b"\x02x", b"bad"), Err(Error::InvalidKey));
    assert_eq!(tree.len(), 3);
    tree.validate().unwrap();
    let memory = tree.into_memory();
    assert!(matches!(
        BTreeMap::open(memory.clone(), Config::default(), Lexicographic),
        Err(Error::Configuration)
    ));
    BTreeMap::open(memory, Config::default(), Framed)
        .unwrap()
        .validate()
        .unwrap();
}

#[derive(Clone, Default)]
struct ObservedMemory {
    memory: VectorMemory,
    reads: Rc<Cell<u64>>,
    deny_growth: Rc<Cell<bool>>,
}
impl Memory for ObservedMemory {
    fn size(&self) -> u64 {
        self.memory.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        if self.deny_growth.get() {
            -1
        } else {
            self.memory.grow(pages)
        }
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        self.reads.set(self.reads.get() + bytes.len() as u64);
        self.memory.read(offset, bytes);
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        self.memory.write(offset, bytes);
    }
}
#[test]
fn icydb_sized_rows_large_keys_and_lazy_values() {
    let memory = ObservedMemory::default();
    let reads = memory.reads.clone();
    let mut tree = BTreeMap::create(memory, Config::default(), Lexicographic).unwrap();
    let row = vec![0x5a; 4 * 1024 * 1024];
    let large_key = vec![0x13; 128 * 1024];
    tree.insert(&large_key, &row).unwrap();
    tree.insert(b"marker", &[0]).unwrap();
    tree.insert(b"", b"").unwrap();
    reads.set(0);
    let keys = tree
        .iter()
        .map(|e| e.unwrap().key().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(keys, vec![vec![], large_key.clone(), b"marker".to_vec()]);
    assert!(
        reads.get() < 256 * 1024,
        "key scan must not read the 4 MiB row"
    );
    assert_eq!(tree.get(&large_key).unwrap().unwrap(), row);
    assert_eq!(
        tree.insert(b"oversize", &vec![0; 4 * 1024 * 1024 + 1]),
        Err(Error::TooLarge)
    );
    tree.validate().unwrap();
    tree.remove(&large_key).unwrap();
    tree.validate().unwrap();
}

#[test]
fn capacity_failure_preserves_all_existing_bytes() {
    let memory = ObservedMemory::default();
    let deny = memory.deny_growth.clone();
    let backing = memory.memory.clone();
    let mut tree = BTreeMap::create(memory, Config::default(), Lexicographic).unwrap();
    tree.insert(b"kept", b"original").unwrap();
    let before = backing.borrow().clone();
    deny.set(true);
    assert_eq!(
        tree.insert(b"kept", &vec![9; 128 * 1024]),
        Err(Error::Capacity)
    );
    assert_eq!(*backing.borrow(), before);
    assert_eq!(tree.get(b"kept").unwrap(), Some(b"original".to_vec()));
    tree.validate().unwrap();
    deny.set(false);
    tree.insert(b"kept", &vec![9; 128 * 1024]).unwrap();
    tree.validate().unwrap();
}

#[test]
fn reject_foreign_corrupt_and_incompatible_memory() {
    let mut tree = fresh();
    tree.insert(b"key", b"value").unwrap();
    let memory = tree.into_memory();
    assert!(matches!(
        Tree::create(memory.clone(), Config::default(), Lexicographic),
        Err(Error::NotEmpty)
    ));
    let limits = Config {
        max_key_bytes: 10,
        ..Config::default()
    };
    assert!(matches!(
        Tree::open(memory.clone(), limits, Lexicographic),
        Err(Error::Configuration)
    ));
    let mut bytes = memory.borrow().clone();
    memory.write(4096 + 100, &[0xff]);
    assert!(matches!(
        Tree::open(memory.clone(), Config::default(), Lexicographic),
        Err(Error::Corrupt)
    ));
    memory.write(0, &bytes);
    bytes[0] ^= 1;
    memory.write(0, &bytes);
    assert!(matches!(
        Tree::open(memory, Config::default(), Lexicographic),
        Err(Error::Format)
    ));
}

#[test]
fn compact_index_markers_do_not_allocate_a_page_per_entry() {
    let mut tree = fresh();
    for n in 0..10000 {
        tree.insert(&key(n), &[0]).unwrap();
    }
    tree.validate().unwrap();
    assert!(tree.memory_bytes() < 4 * 1024 * 1024);
}

#[test]
fn staged_split_is_discarded_when_growth_is_refused() {
    let memory = ObservedMemory::default();
    let backing = memory.memory.clone();
    let deny = memory.deny_growth.clone();
    let mut tree = BTreeMap::create(memory, Config::default(), Lexicographic).unwrap();
    for n in 0..31 {
        tree.insert(&key(n), &[0]).unwrap();
    }
    let before = backing.borrow().clone();
    deny.set(true);
    assert_eq!(
        tree.insert(&key(32), &vec![3; 128 * 1024]),
        Err(Error::Capacity)
    );
    assert_eq!(*backing.borrow(), before);
    assert_eq!(tree.len(), 31);
    tree.validate().unwrap();
}

#[test]
fn corrupt_value_is_lazy_and_failed_mutation_preserves_bytes() {
    let mut tree = fresh();
    let value = vec![0xab; 10000];
    tree.insert(b"key", &value).unwrap();
    let memory = tree.into_memory();
    let payload = memory
        .borrow()
        .windows(64)
        .position(|bytes| bytes == [0xab; 64])
        .unwrap();
    memory.write(payload as u64, &[0x7f]);
    let before = memory.borrow().clone();
    let mut tree = Tree::open(memory.clone(), Config::default(), Lexicographic).unwrap();
    let entry = tree.iter().next().unwrap().unwrap();
    assert_eq!(entry.key(), b"key");
    assert_eq!(entry.value_len(), 10000);
    assert!(tree.contains_key(b"key").unwrap());
    assert!(!tree.contains_key(b"missing").unwrap());
    assert_eq!(entry.value(), Err(Error::Corrupt));
    assert_eq!(tree.remove(b"key"), Err(Error::Corrupt));
    assert_eq!(*memory.borrow(), before);
    assert_eq!(tree.validate(), Err(Error::Corrupt));
}

#[test]
fn iterator_errors_once_and_creation_failure_is_non_destructive() {
    let memory = ObservedMemory::default();
    memory.deny_growth.set(true);
    assert!(matches!(
        BTreeMap::create(memory.clone(), Config::default(), Lexicographic),
        Err(Error::Capacity)
    ));
    assert_eq!(memory.size(), 0);
    memory.deny_growth.set(false);
    let mut tree = BTreeMap::create(memory.clone(), Config::default(), Lexicographic).unwrap();
    tree.insert(b"key", &[1]).unwrap();
    // Deliberately violate exclusive ownership to inject post-open corruption.
    memory.write(4096, &[0xff]);
    let mut range = tree.iter();
    assert!(matches!(range.next(), Some(Err(Error::Corrupt))));
    assert!(range.next_back().is_none());
    assert!(range.next().is_none());
}
