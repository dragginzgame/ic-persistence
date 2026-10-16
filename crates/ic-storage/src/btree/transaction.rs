use super::page::{self, DEGREE, Header, MAX, MAX_HEIGHT, Node, PAYLOAD, Page, Record, SIZE};
use super::{BTreeMap, Comparator, Error, Memory};
use std::{
    cmp::Ordering,
    collections::{BTreeMap as Pages, BTreeSet},
};

pub(super) struct Transaction<'a, M, C> {
    tree: &'a BTreeMap<M, C>,
    pub header: Header,
    pages: Pages<u64, Page>,
    allocated: BTreeSet<u64>,
}
impl<'a, M: Memory, C: Comparator> Transaction<'a, M, C> {
    pub fn new(tree: &'a BTreeMap<M, C>) -> Self {
        Self {
            tree,
            header: tree.header,
            pages: Pages::new(),
            allocated: BTreeSet::new(),
        }
    }
    fn read(&self, id: u64) -> Result<Page, Error> {
        if let Some(page) = self.pages.get(&id) {
            return Ok(*page);
        }
        page::read(&self.tree.memory, self.tree.header.next, id)
    }
    pub fn allocate(&mut self) -> Result<u64, Error> {
        let id;
        if self.header.free == 0 {
            id = self.header.next;
            self.header.next = id.checked_add(1).ok_or(Error::Capacity)?;
        } else {
            id = self.header.free;
            let page = self.read(id)?;
            if page[0] != page::FREE || !self.allocated.insert(id) {
                return Err(Error::Corrupt);
            }
            self.header.free = page::number(&page, 8);
            if self.header.free >= self.header.next {
                return Err(Error::Corrupt);
            }
        }
        Ok(id)
    }
    fn free(&mut self, id: u64) -> Result<(), Error> {
        if id == 0 || id >= self.header.next || self.read(id)?[0] == page::FREE {
            return Err(Error::Corrupt);
        }
        let mut page = [0; SIZE];
        page[0] = page::FREE;
        page::put(&mut page, 8, self.header.free);
        page::seal(&mut page);
        self.pages.insert(id, page);
        self.header.free = id;
        Ok(())
    }
    fn node(&self, id: u64) -> Result<Node, Error> {
        Node::decode(&self.read(id)?, self.header.next, self.header.config)
    }
    pub fn put_node(&mut self, id: u64, node: &Node) -> Result<(), Error> {
        self.pages.insert(id, node.encode()?);
        Ok(())
    }
    fn key(&self, record: Record) -> Result<Vec<u8>, Error> {
        if record.key == 0 {
            return Ok(record.key_inline
                [..usize::try_from(record.key_len).map_err(|_| Error::Corrupt)?]
                .to_vec());
        }
        let mut bytes =
            Vec::with_capacity(usize::try_from(record.key_len).map_err(|_| Error::Corrupt)?);
        let mut id = record.key;
        while bytes.len() < usize::try_from(record.key_len).map_err(|_| Error::Corrupt)? {
            let page = self.read(id)?;
            if page[0] != page::BLOB {
                return Err(Error::Corrupt);
            }
            let take = (usize::try_from(record.key_len).map_err(|_| Error::Corrupt)? - bytes.len())
                .min(PAYLOAD);
            bytes.extend_from_slice(&page[16..16 + take]);
            id = page::number(&page, 8);
        }
        if id != 0 {
            return Err(Error::Corrupt);
        }
        Ok(bytes)
    }
    fn locate(&self, node: &Node, key: &[u8]) -> Result<(usize, bool), Error> {
        let (mut lo, mut hi) = (0, node.entries.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match self
                .tree
                .order
                .compare(&self.key(node.entries[mid])?, key)?
            {
                Ordering::Less => lo = mid + 1,
                Ordering::Equal => return Ok((mid, true)),
                Ordering::Greater => hi = mid,
            }
        }
        Ok((lo, false))
    }
    fn store_blob(&mut self, bytes: &[u8]) -> Result<u64, Error> {
        let mut next = 0;
        for index in (0..bytes.len().div_ceil(PAYLOAD)).rev() {
            let chunk = &bytes[index * PAYLOAD..bytes.len().min((index + 1) * PAYLOAD)];
            let id = self.allocate()?;
            let mut page = [0; SIZE];
            page[0] = page::BLOB;
            page::put(&mut page, 8, next);
            page[16..16 + chunk.len()].copy_from_slice(chunk);
            page::seal(&mut page);
            self.pages.insert(id, page);
            next = id;
        }
        Ok(next)
    }
    fn free_blob(&mut self, mut id: u64, mut len: u64) -> Result<(), Error> {
        if id == 0 {
            return Ok(());
        }
        while len != 0 {
            let page = self.read(id)?;
            if page[0] != page::BLOB {
                return Err(Error::Corrupt);
            }
            let next = page::number(&page, 8);
            self.free(id)?;
            len = len.saturating_sub(PAYLOAD as u64);
            id = next;
        }
        if id != 0 {
            return Err(Error::Corrupt);
        }
        Ok(())
    }
    fn split(&mut self, parent: &mut Node, at: usize) -> Result<(), Error> {
        let left_id = parent.children[at];
        let mut left = self.node(left_id)?;
        if left.entries.len() != MAX {
            return Err(Error::Corrupt);
        }
        let right_entries = left.entries.split_off(DEGREE);
        let median = left.entries.pop().ok_or(Error::Corrupt)?;
        let children = if left.children.is_empty() {
            Vec::new()
        } else {
            left.children.split_off(DEGREE)
        };
        let right_id = self.allocate()?;
        self.put_node(left_id, &left)?;
        self.put_node(
            right_id,
            &Node {
                entries: right_entries,
                children,
            },
        )?;
        parent.entries.insert(at, median);
        parent.children.insert(at + 1, right_id);
        Ok(())
    }
    fn replace_value(&mut self, record: &mut Record, value: &[u8]) -> Result<(), Error> {
        let mut ids = Vec::new();
        let mut id = record.value;
        let mut remaining = record.value_len;
        if id != 0 {
            while remaining != 0 {
                let page = self.read(id)?;
                if page[0] != page::BLOB {
                    return Err(Error::Corrupt);
                }
                ids.push(id);
                id = page::number(&page, 8);
                remaining = remaining.saturating_sub(PAYLOAD as u64);
            }
            if id != 0 {
                return Err(Error::Corrupt);
            }
        }
        let needed = if value.len() <= 16 {
            0
        } else {
            value.len().div_ceil(PAYLOAD)
        };
        while ids.len() > needed {
            self.free(ids.pop().ok_or(Error::Corrupt)?)?;
        }
        while ids.len() < needed {
            ids.push(self.allocate()?);
        }
        for (index, id) in ids.iter().copied().enumerate() {
            let chunk = &value[index * PAYLOAD..value.len().min((index + 1) * PAYLOAD)];
            let mut page = [0; SIZE];
            page[0] = page::BLOB;
            page::put(&mut page, 8, ids.get(index + 1).copied().unwrap_or(0));
            page[16..16 + chunk.len()].copy_from_slice(chunk);
            page::seal(&mut page);
            self.pages.insert(id, page);
        }
        record.value = ids.first().copied().unwrap_or(0);
        record.value_len = value.len() as u64;
        record.value_inline = [0; 16];
        if needed == 0 {
            record.value_inline[..value.len()].copy_from_slice(value);
        }
        Ok(())
    }
    pub fn replace(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let mut id = self.header.root;
        for _ in 0..MAX_HEIGHT {
            let mut node = self.node(id)?;
            let (at, found) = self.locate(&node, key)?;
            if found {
                self.replace_value(&mut node.entries[at], value)?;
                return self.put_node(id, &node);
            }
            id = *node.children.get(at).ok_or(Error::Corrupt)?;
        }
        Err(Error::Corrupt)
    }
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let root = self.node(self.header.root)?;
        if root.entries.len() == MAX {
            let id = self.allocate()?;
            let mut parent = Node {
                entries: Vec::new(),
                children: vec![self.header.root],
            };
            self.split(&mut parent, 0)?;
            self.put_node(id, &parent)?;
            self.header.root = id;
        }
        let mut id = self.header.root;
        for _ in 0..MAX_HEIGHT {
            let mut node = self.node(id)?;
            let (mut at, found) = self.locate(&node, key)?;
            if found {
                return Err(Error::Corrupt);
            }
            if node.children.is_empty() {
                let mut key_inline = [0; 32];
                let mut value_inline = [0; 16];
                if key.len() <= 32 {
                    key_inline[..key.len()].copy_from_slice(key);
                }
                if value.len() <= 16 {
                    value_inline[..value.len()].copy_from_slice(value);
                }
                let record = Record {
                    key_inline,
                    value_inline,
                    key: if key.len() <= 32 {
                        0
                    } else {
                        self.store_blob(key)?
                    },
                    key_len: key.len() as u64,
                    value: if value.len() <= 16 {
                        0
                    } else {
                        self.store_blob(value)?
                    },
                    value_len: value.len() as u64,
                };
                node.entries.insert(at, record);
                return self.put_node(id, &node);
            }
            if self.node(node.children[at])?.entries.len() == MAX {
                self.split(&mut node, at)?;
                let comparison = self.tree.order.compare(key, &self.key(node.entries[at])?)?;
                self.put_node(id, &node)?;
                if comparison == Ordering::Equal {
                    return Err(Error::Corrupt);
                }
                if comparison == Ordering::Greater {
                    at += 1;
                }
            }
            id = node.children[at];
        }
        Err(Error::Corrupt)
    }
    // Merge adjacent children around their parent separator; return the survivor.
    fn merge(&mut self, parent: &mut Node, at: usize) -> Result<u64, Error> {
        let left_id = parent.children[at];
        let right_id = parent.children.remove(at + 1);
        let mut left = self.node(left_id)?;
        let right = self.node(right_id)?;
        if left.children.is_empty() != right.children.is_empty() {
            return Err(Error::Corrupt);
        }
        left.entries.push(parent.entries.remove(at));
        left.entries.extend(right.entries);
        left.children.extend(right.children);
        self.put_node(left_id, &left)?;
        self.free(right_id)?;
        Ok(left_id)
    }
    fn prepare_child(&mut self, parent: &mut Node, at: usize) -> Result<u64, Error> {
        let id = parent.children[at];
        let mut child = self.node(id)?;
        if child.entries.len() >= DEGREE {
            return Ok(id);
        }
        if child.entries.len() != DEGREE - 1 {
            return Err(Error::Corrupt);
        }
        if at > 0 {
            let left_id = parent.children[at - 1];
            let mut left = self.node(left_id)?;
            if left.entries.len() >= DEGREE {
                child.entries.insert(0, parent.entries[at - 1]);
                parent.entries[at - 1] = left.entries.pop().ok_or(Error::Corrupt)?;
                if !left.children.is_empty() {
                    child
                        .children
                        .insert(0, left.children.pop().ok_or(Error::Corrupt)?);
                }
                self.put_node(left_id, &left)?;
                self.put_node(id, &child)?;
                return Ok(id);
            }
        }
        if at + 1 < parent.children.len() {
            let right_id = parent.children[at + 1];
            let mut right = self.node(right_id)?;
            if right.entries.len() >= DEGREE {
                child.entries.push(parent.entries[at]);
                parent.entries[at] = right.entries.remove(0);
                if !right.children.is_empty() {
                    child.children.push(right.children.remove(0));
                }
                self.put_node(right_id, &right)?;
                self.put_node(id, &child)?;
                return Ok(id);
            }
            return self.merge(parent, at);
        }
        self.merge(parent, at.checked_sub(1).ok_or(Error::Corrupt)?)
    }
    fn extreme(&self, mut id: u64, last: bool) -> Result<Record, Error> {
        for _ in 0..MAX_HEIGHT {
            let node = self.node(id)?;
            if node.children.is_empty() {
                return (if last {
                    node.entries.last()
                } else {
                    node.entries.first()
                })
                .copied()
                .ok_or(Error::Corrupt);
            }
            id = if last {
                *node.children.last().ok_or(Error::Corrupt)?
            } else {
                node.children[0]
            };
        }
        Err(Error::Corrupt)
    }
    fn delete(&mut self, id: u64, key: &[u8], depth: usize) -> Result<Record, Error> {
        if depth >= MAX_HEIGHT {
            return Err(Error::Corrupt);
        }
        let mut node = self.node(id)?;
        let (at, found) = self.locate(&node, key)?;
        let removed;
        if found {
            removed = node.entries[at];
            if node.children.is_empty() {
                node.entries.remove(at);
            } else if self.node(node.children[at])?.entries.len() >= DEGREE {
                let replacement = self.extreme(node.children[at], true)?;
                self.delete(node.children[at], &self.key(replacement)?, depth + 1)?;
                node.entries[at] = replacement;
            } else if self.node(node.children[at + 1])?.entries.len() >= DEGREE {
                let replacement = self.extreme(node.children[at + 1], false)?;
                self.delete(node.children[at + 1], &self.key(replacement)?, depth + 1)?;
                node.entries[at] = replacement;
            } else {
                let child = self.merge(&mut node, at)?;
                self.delete(child, key, depth + 1)?;
            }
        } else {
            if node.children.is_empty() {
                return Err(Error::Corrupt);
            }
            let child = self.prepare_child(&mut node, at)?;
            removed = self.delete(child, key, depth + 1)?;
        }
        self.put_node(id, &node)?;
        Ok(removed)
    }
    pub fn remove(&mut self, key: &[u8]) -> Result<(), Error> {
        let removed = self.delete(self.header.root, key, 0)?;
        self.free_blob(removed.key, removed.key_len)?;
        self.free_blob(removed.value, removed.value_len)?;
        let root = self.node(self.header.root)?;
        if root.entries.is_empty() && !root.children.is_empty() {
            self.free(self.header.root)?;
            self.header.root = root.children[0];
        }
        Ok(())
    }
    pub fn commit(self) -> Result<Header, Error> {
        let bytes = self
            .header
            .next
            .checked_mul(SIZE as u64)
            .ok_or(Error::Capacity)?;
        let required = bytes.div_ceil(65_536);
        let current = self.tree.memory.size();
        if required > current && self.tree.memory.grow(required - current) < 0 {
            return Err(Error::Capacity);
        }
        // No fallible application operation follows the first write. A backing
        // trap must roll the entire message back; filesystem crash atomicity is
        // intentionally not claimed by this storage contract.
        for (id, page) in self.pages {
            self.tree.memory.write(id * SIZE as u64, &page);
        }
        self.tree.memory.write(0, &self.header.encode());
        Ok(self.header)
    }
}

pub(super) fn validate<M: Memory, C: Comparator>(tree: &BTreeMap<M, C>) -> Result<(), Error> {
    let mut seen = BTreeSet::new();
    let mut previous = None;
    let mut count = 0u64;
    let mut leaf_depth = None;
    audit_node(
        tree,
        tree.header.root,
        0,
        &mut seen,
        &mut previous,
        &mut count,
        &mut leaf_depth,
    )?;
    if count != tree.header.len {
        return Err(Error::Corrupt);
    }
    let mut free = tree.header.free;
    while free != 0 {
        if !seen.insert(free) {
            return Err(Error::Corrupt);
        }
        let page = page::read(&tree.memory, tree.header.next, free)?;
        if page[0] != page::FREE {
            return Err(Error::Corrupt);
        }
        free = page::number(&page, 8);
    }
    if seen.len() as u64 != tree.header.next - 1 {
        return Err(Error::Corrupt);
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn audit_node<M: Memory, C: Comparator>(
    tree: &BTreeMap<M, C>,
    id: u64,
    depth: usize,
    seen: &mut BTreeSet<u64>,
    previous: &mut Option<Vec<u8>>,
    count: &mut u64,
    leaf_depth: &mut Option<usize>,
) -> Result<(), Error> {
    if depth >= MAX_HEIGHT || !seen.insert(id) {
        return Err(Error::Corrupt);
    }
    let node = tree.node(id)?;
    if depth > 0 && node.entries.len() < DEGREE - 1 {
        return Err(Error::Corrupt);
    }
    if node.entries.is_empty() && !node.children.is_empty() {
        return Err(Error::Corrupt);
    }
    if node.children.is_empty() {
        if leaf_depth.is_some_and(|expected| expected != depth) {
            return Err(Error::Corrupt);
        }
        *leaf_depth = Some(depth);
    }
    for (i, record) in node.entries.iter().enumerate() {
        if !node.children.is_empty() {
            audit_node(
                tree,
                node.children[i],
                depth + 1,
                seen,
                previous,
                count,
                leaf_depth,
            )?;
        }
        audit_blob(tree, record.key, record.key_len, seen)?;
        audit_blob(tree, record.value, record.value_len, seen)?;
        let key = tree.key(*record)?;
        tree.check_key(&key)?;
        if let Some(previous) = previous.as_ref() {
            if tree.order.compare(previous, &key)? != Ordering::Less {
                return Err(Error::Corrupt);
            }
        }
        *previous = Some(key);
        *count = count.checked_add(1).ok_or(Error::Corrupt)?;
    }
    if let Some(last) = node.children.last() {
        audit_node(tree, *last, depth + 1, seen, previous, count, leaf_depth)?;
    }
    Ok(())
}
fn audit_blob<M: Memory, C: Comparator>(
    tree: &BTreeMap<M, C>,
    mut id: u64,
    mut len: u64,
    seen: &mut BTreeSet<u64>,
) -> Result<(), Error> {
    if id == 0 {
        return Ok(());
    }
    while len != 0 {
        if !seen.insert(id) {
            return Err(Error::Corrupt);
        }
        let page = page::read(&tree.memory, tree.header.next, id)?;
        if page[0] != page::BLOB {
            return Err(Error::Corrupt);
        }
        id = page::number(&page, 8);
        len = len.saturating_sub(PAYLOAD as u64);
    }
    if id != 0 {
        return Err(Error::Corrupt);
    }
    Ok(())
}
