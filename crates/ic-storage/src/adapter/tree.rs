//! Owned AVL ordered storage. Small mutations stage affected nodes and allocator
//! metadata, reserve capacity once, then publish the protected root last.
//! Power-of-two reusable blocks bound churn without whole-map heap snapshots.
use super::{
    Memory, Storable,
    error::{CollectionError as Error, Result, ensure},
    record,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::BTreeMap,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

pub const MAGIC: [u8; 8] = *b"ICMAP001";
const NODE: [u8; 8] = *b"ICNODE01";
const FREE: [u8; 8] = *b"ICFREE01";
const BINS: usize = 32;
const MAX_DEPTH: usize = 96;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct Pointer {
    offset: u64,
    order: u8,
}
impl Pointer {
    fn capacity(self) -> u64 {
        1_u64 << self.order
    }
    fn check(self, cursor: u64) -> Result<()> {
        ensure(
            (8..32).contains(&self.order),
            Error::Corrupt("tree block order"),
        )?;
        ensure(
            self.offset >= record::HEADER_BYTES
                && self
                    .offset
                    .checked_add(self.capacity())
                    .is_some_and(|end| end <= cursor),
            Error::Corrupt("tree block bounds"),
        )?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Root {
    top: Option<Pointer>,
    length: u64,
    cursor: u64,
    free: Vec<Option<Pointer>>,
    key_bound: Option<(u32, bool)>,
    value_bound: Option<(u32, bool)>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Node {
    left: Option<Pointer>,
    right: Option<Pointer>,
    height: u8,
    key: Vec<u8>,
    value: Vec<u8>,
}

pub struct Tree<K, V, M: Memory> {
    pub memory: M,
    root: RefCell<Root>,
    poisoned: Cell<bool>,
    types: PhantomData<(K, V)>,
}

impl<K: Storable + Ord + Clone, V: Storable, M: Memory> Tree<K, V, M> {
    pub fn reset(memory: M) -> Result<Self> {
        let root = Root {
            top: None,
            length: 0,
            cursor: record::HEADER_BYTES,
            free: vec![None; BINS],
            key_bound: record::bound::<K>(),
            value_bound: record::bound::<V>(),
        };
        let bytes = record::encode(MAGIC, &root)?;
        ensure(
            bytes.len() as u64 <= record::HEADER_BYTES,
            Error::Corrupt("map invariant"),
        )?;
        record::reserve(&memory, record::HEADER_BYTES)?;
        memory.write(0, &bytes);
        Ok(Self {
            memory,
            root: RefCell::new(root),
            poisoned: Cell::new(false),
            types: PhantomData,
        })
    }

    pub fn load(memory: M) -> Result<Self> {
        let root: Root = record::read(&memory, 0, record::HEADER_BYTES, MAGIC)?;
        ensure(
            root.key_bound == record::bound::<K>(),
            Error::Corrupt("map key bound changed"),
        )?;
        ensure(
            root.value_bound == record::bound::<V>(),
            Error::Corrupt("map value bound changed"),
        )?;
        ensure(
            root.free.len() == BINS
                && root.cursor >= record::HEADER_BYTES
                && root.cursor <= memory.size() * crate::WASM_PAGE_SIZE_BYTES,
            Error::Corrupt("map allocator bounds"),
        )?;
        ensure(
            root.top.is_none() == (root.length == 0),
            Error::Corrupt("map root/count disagree"),
        )?;
        if let Some(pointer) = root.top {
            pointer.check(root.cursor)?;
        }
        for (order, pointer) in root.free.iter().enumerate() {
            if let Some(pointer) = pointer {
                pointer.check(root.cursor)?;
                ensure(
                    usize::from(pointer.order) == order,
                    Error::Corrupt("free block order"),
                )?;
            }
        }
        Ok(Self {
            memory,
            root: RefCell::new(root),
            poisoned: Cell::new(false),
            types: PhantomData,
        })
    }

    fn healthy(&self) -> Result<()> {
        ensure(!self.poisoned.get(), Error::Poisoned)
    }
    pub fn len(&self) -> u64 {
        self.healthy().expect("map owner is poisoned");
        self.root.borrow().length
    }
    fn node(&self, pointer: Pointer) -> Result<Node> {
        pointer.check(self.root.borrow().cursor)?;
        let node: Node = record::read(&self.memory, pointer.offset, pointer.capacity(), NODE)?;
        validate_node::<K, V>(&node, self.root.borrow().cursor)?;
        Ok(node)
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        self.healthy()?;
        let mut current = self.root.borrow().top;
        for _ in 0..MAX_DEPTH {
            let Some(pointer) = current else {
                return Ok(None);
            };
            let node = self.node(pointer)?;
            match key.cmp(&decode::<K>(node.key)?) {
                Ordering::Less => current = node.left,
                Ordering::Greater => current = node.right,
                Ordering::Equal => return decode(node.value).map(Some),
            }
        }
        Err(Error::Corrupt("map path exceeds height bound"))
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        self.healthy()?;
        let key_bytes = key.to_bytes_checked()?.into_owned();
        let value_bytes = value.into_bytes_checked()?;
        let mut transaction = Transaction::<K, V, M>::new(self);
        let (root, previous) =
            transaction.insert(transaction.root.top, &key, &key_bytes, &value_bytes, 0)?;
        transaction.root.top = Some(root);
        drop(key);
        let previous = previous.map(decode).transpose()?;
        transaction.commit()?;
        Ok(previous)
    }

    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        self.healthy()?;
        let mut transaction = Transaction::<K, V, M>::new(self);
        let (root, previous) = transaction.remove(transaction.root.top, key, 0)?;
        transaction.root.top = root;
        let previous = previous.map(decode).transpose()?;
        if previous.is_some() {
            transaction.commit()?;
        }
        Ok(previous)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.healthy()?;
        let mut transaction = Transaction::<K, V, M>::new(self);
        transaction.root.top = None;
        transaction.root.length = 0;
        transaction.root.cursor = record::HEADER_BYTES;
        transaction.root.free.fill(None);
        transaction.commit()
    }

    fn seek_path(&self, bound: &Bound<K>, forward: bool) -> Result<Vec<Pointer>> {
        self.healthy()?;
        let mut current = self.root.borrow().top;
        let mut path = Vec::new();
        for _ in 0..MAX_DEPTH {
            let Some(pointer) = current else {
                return Ok(path);
            };
            let node = self.node(pointer)?;
            let key = decode::<K>(node.key)?;
            let accepted = match bound {
                Bound::Unbounded => true,
                Bound::Included(limit) => {
                    if forward {
                        key >= *limit
                    } else {
                        key <= *limit
                    }
                }
                Bound::Excluded(limit) => {
                    if forward {
                        key > *limit
                    } else {
                        key < *limit
                    }
                }
            };
            if accepted {
                current = if forward { node.left } else { node.right };
                path.push(pointer);
            } else {
                current = if forward { node.right } else { node.left };
            }
        }
        Err(Error::Corrupt("map path exceeds height bound"))
    }

    fn advance_path(
        &self,
        path: &mut Vec<Pointer>,
        forward: bool,
    ) -> Result<Option<MapEntry<K, V>>> {
        self.healthy()?;
        let Some(pointer) = path.pop() else {
            return Ok(None);
        };
        let node = self.node(pointer)?;
        let mut child = if forward { node.right } else { node.left };
        let mut depth = 0;
        while let Some(pointer) = child {
            ensure(
                depth < MAX_DEPTH && path.len() < MAX_DEPTH,
                Error::Corrupt("map cursor exceeds height bound"),
            )?;
            depth += 1;
            path.push(pointer);
            let child_node = self.node(pointer)?;
            child = if forward {
                child_node.left
            } else {
                child_node.right
            };
        }
        Ok(Some(MapEntry {
            key: decode(node.key)?,
            bytes: node.value,
            value: PhantomData,
        }))
    }

    pub fn range(&self, bounds: impl RangeBounds<K>) -> TreeIter<'_, K, V, M> {
        let lower = bounds.start_bound().cloned();
        let upper = bounds.end_bound().cloned();
        let invalid = match (&lower, &upper) {
            (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) => {
                a > b
                    || (a == b
                        && matches!((&lower, &upper), (Bound::Excluded(_), Bound::Excluded(_))))
            }
            _ => false,
        };
        TreeIter {
            tree: self,
            lower,
            upper,
            front: None,
            back: None,
            exhausted: false,
            invalid,
        }
    }
}

fn decode<T: Storable>(bytes: Vec<u8>) -> Result<T> {
    T::check_bounds(&bytes)?;
    Ok(T::from_bytes(Cow::Owned(bytes))?)
}
fn validate_node<K: Storable, V: Storable>(node: &Node, cursor: u64) -> Result<()> {
    ensure(
        (1..=96).contains(&node.height),
        Error::Corrupt("map node height"),
    )?;
    K::check_bounds(&node.key)?;
    V::check_bounds(&node.value)?;
    for pointer in [node.left, node.right].into_iter().flatten() {
        pointer.check(cursor)?;
    }
    Ok(())
}

struct Transaction<'a, K, V, M: Memory> {
    tree: &'a Tree<K, V, M>,
    root: Root,
    writes: BTreeMap<u64, Vec<u8>>,
}

impl<'a, K: Storable + Ord + Clone, V: Storable, M: Memory> Transaction<'a, K, V, M> {
    fn new(tree: &'a Tree<K, V, M>) -> Self {
        Self {
            tree,
            root: tree.root.borrow().clone(),
            writes: BTreeMap::new(),
        }
    }
    fn read<T: serde::de::DeserializeOwned>(&self, pointer: Pointer, magic: [u8; 8]) -> Result<T> {
        pointer.check(self.root.cursor)?;
        if let Some(bytes) = self.writes.get(&pointer.offset) {
            record::decode(magic, bytes)
        } else {
            record::read(&self.tree.memory, pointer.offset, pointer.capacity(), magic)
        }
    }
    fn node(&self, pointer: Pointer) -> Result<Node> {
        let node: Node = self.read(pointer, NODE)?;
        validate_node::<K, V>(&node, self.root.cursor)?;
        Ok(node)
    }
    fn height(&self, pointer: Option<Pointer>) -> Result<u8> {
        pointer.map_or(Ok(0), |pointer| Ok(self.node(pointer)?.height))
    }
    fn free(&mut self, pointer: Pointer) -> Result<()> {
        let index = usize::from(pointer.order);
        self.writes.insert(
            pointer.offset,
            record::encode(FREE, &self.root.free[index])?,
        );
        self.root.free[index] = Some(pointer);
        Ok(())
    }
    fn allocate(&mut self, length: usize) -> Result<Pointer> {
        let capacity = length
            .max(256)
            .checked_next_power_of_two()
            .ok_or(Error::Budget)?;
        let order = usize::try_from(capacity.trailing_zeros()).unwrap();
        ensure(order < BINS, Error::Corrupt("map record too large"))?;
        for bin in order..BINS {
            if let Some(pointer) = self.root.free[bin] {
                let next: Option<Pointer> = self.read(pointer, FREE)?;
                if let Some(next) = next {
                    next.check(self.root.cursor)?;
                    ensure(
                        next.order == pointer.order,
                        Error::Corrupt("free block order"),
                    )?;
                    ensure(next != pointer, Error::Corrupt("map free-list cycle"))?;
                }
                self.root.free[bin] = next;
                return Ok(pointer);
            }
        }
        let pointer = Pointer {
            offset: self.root.cursor,
            order: u8::try_from(order).unwrap(),
        };
        self.root.cursor = self
            .root
            .cursor
            .checked_add(pointer.capacity())
            .ok_or(Error::Budget)?;
        Ok(pointer)
    }
    fn save(&mut self, previous: Option<Pointer>, node: &Node) -> Result<Pointer> {
        let bytes = record::encode(NODE, node)?;
        let pointer = if let Some(pointer) =
            previous.filter(|pointer| pointer.capacity() >= bytes.len() as u64)
        {
            pointer
        } else {
            if let Some(pointer) = previous {
                self.free(pointer)?;
            }
            self.allocate(bytes.len())?
        };
        self.writes.insert(pointer.offset, bytes);
        Ok(pointer)
    }
    fn rotate_left(&mut self, pointer: Pointer, mut node: Node) -> Result<Pointer> {
        let right_pointer = node.right.ok_or(Error::Corrupt("map rotation child"))?;
        let mut right = self.node(right_pointer)?;
        node.right = right.left;
        node.height = 1 + self.height(node.left)?.max(self.height(node.right)?);
        right.left = Some(self.save(Some(pointer), &node)?);
        right.height = 1 + self.height(right.left)?.max(self.height(right.right)?);
        self.save(Some(right_pointer), &right)
    }
    fn rotate_right(&mut self, pointer: Pointer, mut node: Node) -> Result<Pointer> {
        let left_pointer = node.left.ok_or(Error::Corrupt("map rotation child"))?;
        let mut left = self.node(left_pointer)?;
        node.left = left.right;
        node.height = 1 + self.height(node.left)?.max(self.height(node.right)?);
        left.right = Some(self.save(Some(pointer), &node)?);
        left.height = 1 + self.height(left.left)?.max(self.height(left.right)?);
        self.save(Some(left_pointer), &left)
    }
    fn balance(&mut self, pointer: Pointer, mut node: Node) -> Result<Pointer> {
        let left = self.height(node.left)?;
        let right = self.height(node.right)?;
        node.height = 1 + left.max(right);
        if i16::from(left) - i16::from(right) > 1 {
            let child = node.left.unwrap();
            let value = self.node(child)?;
            if self.height(value.right)? > self.height(value.left)? {
                node.left = Some(self.rotate_left(child, value)?);
            }
            self.rotate_right(pointer, node)
        } else if i16::from(right) - i16::from(left) > 1 {
            let child = node.right.unwrap();
            let value = self.node(child)?;
            if self.height(value.left)? > self.height(value.right)? {
                node.right = Some(self.rotate_right(child, value)?);
            }
            self.rotate_left(pointer, node)
        } else {
            self.save(Some(pointer), &node)
        }
    }
    fn insert(
        &mut self,
        pointer: Option<Pointer>,
        key: &K,
        key_bytes: &[u8],
        value: &[u8],
        depth: usize,
    ) -> Result<(Pointer, Option<Vec<u8>>)> {
        ensure(
            depth < MAX_DEPTH,
            Error::Corrupt("map path exceeds height bound"),
        )?;
        let Some(pointer) = pointer else {
            let node = Node {
                left: None,
                right: None,
                height: 1,
                key: key_bytes.to_vec(),
                value: value.to_vec(),
            };
            self.root.length = self.root.length.checked_add(1).ok_or(Error::Budget)?;
            return Ok((self.save(None, &node)?, None));
        };
        let mut node = self.node(pointer)?;
        let previous = match key.cmp(&decode::<K>(node.key.clone())?) {
            Ordering::Less => {
                let (child, old) = self.insert(node.left, key, key_bytes, value, depth + 1)?;
                if old.is_some() && node.left == Some(child) {
                    return Ok((pointer, old));
                }
                node.left = Some(child);
                old
            }
            Ordering::Greater => {
                let (child, old) = self.insert(node.right, key, key_bytes, value, depth + 1)?;
                if old.is_some() && node.right == Some(child) {
                    return Ok((pointer, old));
                }
                node.right = Some(child);
                old
            }
            Ordering::Equal => Some(std::mem::replace(&mut node.value, value.to_vec())),
        };
        // Replacement can relocate a record, but never changes subtree height.
        let pointer = if previous.is_some() {
            self.save(Some(pointer), &node)?
        } else {
            self.balance(pointer, node)?
        };
        Ok((pointer, previous))
    }
    fn remove(
        &mut self,
        pointer: Option<Pointer>,
        key: &K,
        depth: usize,
    ) -> Result<(Option<Pointer>, Option<Vec<u8>>)> {
        ensure(
            depth < MAX_DEPTH,
            Error::Corrupt("map path exceeds height bound"),
        )?;
        let Some(pointer) = pointer else {
            return Ok((None, None));
        };
        let mut node = self.node(pointer)?;
        let previous = match key.cmp(&decode::<K>(node.key.clone())?) {
            Ordering::Less => {
                let (child, old) = self.remove(node.left, key, depth + 1)?;
                node.left = child;
                old
            }
            Ordering::Greater => {
                let (child, old) = self.remove(node.right, key, depth + 1)?;
                node.right = child;
                old
            }
            Ordering::Equal => {
                if node.left.is_none() || node.right.is_none() {
                    self.free(pointer)?;
                    self.root.length = self
                        .root
                        .length
                        .checked_sub(1)
                        .ok_or(Error::Corrupt("map length"))?;
                    return Ok((node.left.or(node.right), Some(node.value)));
                }
                let (right, successor) = self.extract_min(node.right.unwrap(), depth + 1)?;
                node.right = right;
                node.key = successor.key;
                Some(std::mem::replace(&mut node.value, successor.value))
            }
        };
        if previous.is_none() {
            Ok((Some(pointer), None))
        } else {
            Ok((Some(self.balance(pointer, node)?), previous))
        }
    }
    fn extract_min(&mut self, pointer: Pointer, depth: usize) -> Result<(Option<Pointer>, Node)> {
        ensure(
            depth < MAX_DEPTH,
            Error::Corrupt("map path exceeds height bound"),
        )?;
        let mut node = self.node(pointer)?;
        let Some(left) = node.left else {
            self.free(pointer)?;
            self.root.length = self
                .root
                .length
                .checked_sub(1)
                .ok_or(Error::Corrupt("map length"))?;
            return Ok((node.right, node));
        };
        let (child, minimum) = self.extract_min(left, depth + 1)?;
        node.left = child;
        Ok((Some(self.balance(pointer, node)?), minimum))
    }
    fn commit(self) -> Result<()> {
        let header = record::encode(MAGIC, &self.root)?;
        ensure(
            header.len() as u64 <= record::HEADER_BYTES,
            Error::Corrupt("map invariant"),
        )?;
        record::reserve(&self.tree.memory, self.root.cursor)?;
        self.tree.poisoned.set(true);
        for (offset, bytes) in self.writes {
            self.tree.memory.write(offset, &bytes);
        }
        self.tree.memory.write(0, &header);
        *self.tree.root.borrow_mut() = self.root;
        self.tree.poisoned.set(false);
        Ok(())
    }
}

///
/// `MapEntry`
///
/// An owned key and lazily decoded value from an ordered scan.
///
pub struct MapEntry<K, V> {
    key: K,
    bytes: Vec<u8>,
    value: PhantomData<V>,
}
impl<K, V: Storable> MapEntry<K, V> {
    /// Borrow the decoded key.
    pub const fn key(&self) -> &K {
        &self.key
    }
    /// Decode the value using its declared codec.
    ///
    /// # Errors
    /// Returns the application codec or declared-bound error for this value.
    pub fn value(&self) -> Result<V> {
        decode(self.bytes.clone())
    }
    /// Consume the entry and decode its value.
    ///
    /// # Errors
    /// Returns the application codec or declared-bound error for this value.
    pub fn into_pair(self) -> Result<(K, V)> {
        Ok((self.key, decode(self.bytes)?))
    }
    /// Consume the entry without decoding its value.
    pub fn into_key(self) -> K {
        self.key
    }
    /// Consume the entry and decode only its value.
    ///
    /// # Errors
    /// Returns the application codec or declared-bound error for this value.
    pub fn into_value(self) -> Result<V> {
        decode(self.bytes)
    }
}

///
/// `TreeIter`
///
/// Lazy bidirectional traversal with bounded pointer stacks. Ancestor payloads
/// are not retained; values are decoded only when requested by the caller.
///
pub struct TreeIter<'a, K, V, M: Memory> {
    tree: &'a Tree<K, V, M>,
    lower: Bound<K>,
    upper: Bound<K>,
    front: Option<Vec<Pointer>>,
    back: Option<Vec<Pointer>>,
    exhausted: bool,
    invalid: bool,
}
impl<K: Storable + Ord + Clone, V: Storable, M: Memory> TreeIter<'_, K, V, M> {
    fn step(&mut self, forward: bool) -> Result<Option<MapEntry<K, V>>> {
        ensure(!self.invalid, Error::Bounds)?;
        let (path, bound) = if forward {
            (&mut self.front, &self.lower)
        } else {
            (&mut self.back, &self.upper)
        };
        if path.is_none() {
            *path = Some(self.tree.seek_path(bound, forward)?);
        }
        let entry = self.tree.advance_path(path.as_mut().unwrap(), forward)?;
        if let Some(entry) = &entry {
            let after = match &self.lower {
                Bound::Unbounded => true,
                Bound::Included(k) => entry.key >= *k,
                Bound::Excluded(k) => entry.key > *k,
            };
            let before = match &self.upper {
                Bound::Unbounded => true,
                Bound::Included(k) => entry.key <= *k,
                Bound::Excluded(k) => entry.key < *k,
            };
            ensure(
                if forward { after } else { before },
                Error::Corrupt("map cursor key order"),
            )?;
            if !(after && before) {
                return Ok(None);
            }
            if forward {
                self.lower = Bound::Excluded(entry.key.clone());
            } else {
                self.upper = Bound::Excluded(entry.key.clone());
            }
        }
        Ok(entry)
    }
    fn advance(&mut self, forward: bool) -> Option<Result<MapEntry<K, V>>> {
        if self.exhausted {
            return None;
        }
        match self.step(forward) {
            Ok(Some(entry)) => Some(Ok(entry)),
            Ok(None) => {
                self.exhausted = true;
                None
            }
            Err(error) => {
                self.exhausted = true;
                Some(Err(error))
            }
        }
    }
}
impl<K: Storable + Ord + Clone, V: Storable, M: Memory> Iterator for TreeIter<'_, K, V, M> {
    type Item = Result<MapEntry<K, V>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.advance(true)
    }
}
impl<K: Storable + Ord + Clone, V: Storable, M: Memory> DoubleEndedIterator
    for TreeIter<'_, K, V, M>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.advance(false)
    }
}
impl<K: Storable + Ord + Clone, V: Storable, M: Memory> std::iter::FusedIterator
    for TreeIter<'_, K, V, M>
{
}
