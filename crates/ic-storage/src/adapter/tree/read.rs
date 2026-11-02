//! Checked read views retain one record buffer. The optional cache is heap-only.
use super::{
    Entry, Error, INLINE, MAX_ENTRIES, NODE, Node, Pointer, Reader, Result, Storable, Value,
    ensure, record,
};
use std::{borrow::Cow, cmp::Ordering, ops::Range, rc::Rc};

struct ReadEntry {
    key: Range<usize>,
    value: ReadValue,
}
enum ReadValue {
    Inline(Range<usize>),
    External(Pointer),
}
pub(super) struct ReadNode {
    bytes: Vec<u8>,
    entries: Vec<ReadEntry>,
    pub children: Vec<Pointer>,
}
impl ReadNode {
    pub fn decode<K: Storable, V: Storable>(bytes: Vec<u8>, cursor: u64) -> Result<Self> {
        let payload = record::decode_payload(NODE, &bytes)?;
        let mut r = Reader(payload);
        let count = usize::from(r.byte()?);
        let branch = r.byte()?;
        ensure(
            count <= MAX_ENTRIES && branch <= 1,
            Error::Corrupt("B-tree node header"),
        )?;
        let mut children = Vec::with_capacity(if branch == 1 { count + 1 } else { 0 });
        if branch == 1 {
            for _ in 0..=count {
                children.push(r.pointer(cursor)?);
            }
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let len = u32::from_le_bytes(r.take(4)?.try_into().unwrap()) as usize;
            let start = bytes.len() - r.0.len();
            K::check_bounds(r.take(len)?)?;
            let key = start..start + len;
            let size = r.byte()?;
            let value = if size == 255 {
                ReadValue::External(r.pointer(cursor)?)
            } else {
                ensure(
                    usize::from(size) <= INLINE,
                    Error::Corrupt("B-tree inline value"),
                )?;
                let start = bytes.len() - r.0.len();
                V::check_bounds(r.take(usize::from(size))?)?;
                ReadValue::Inline(start..start + usize::from(size))
            };
            entries.push(ReadEntry { key, value });
        }
        ensure(r.0.is_empty(), Error::Corrupt("B-tree node trailing bytes"))?;
        Ok(Self {
            bytes,
            entries,
            children,
        })
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn search<K: Storable + Ord>(&self, key: &K) -> Result<std::result::Result<usize, usize>> {
        let (mut low, mut high) = (0, self.len());
        while low < high {
            let mid = low + (high - low) / 2;
            match key.cmp(&K::from_bytes(Cow::Borrowed(
                &self.bytes[self.entries[mid].key.clone()],
            ))?) {
                Ordering::Less => high = mid,
                Ordering::Greater => low = mid + 1,
                Ordering::Equal => return Ok(Ok(mid)),
            }
        }
        Ok(Err(low))
    }
    pub fn value(&self, index: usize) -> Value {
        match &self.entries[index].value {
            ReadValue::Inline(range) => Value::Inline(self.bytes[range.clone()].to_vec()),
            ReadValue::External(pointer) => Value::External(*pointer),
        }
    }
    pub fn entry(&self, index: usize) -> Entry {
        Entry {
            key: self.bytes[self.entries[index].key.clone()].to_vec(),
            value: self.value(index),
        }
    }
    pub fn owned(&self) -> Node {
        Node {
            entries: (0..self.len()).map(|i| self.entry(i)).collect(),
            children: self.children.clone(),
        }
    }
    fn heap_bytes(&self) -> usize {
        size_of::<Self>()
            + 2 * size_of::<usize>()
            + self.bytes.capacity()
            + self.entries.capacity() * size_of::<ReadEntry>()
            + self.children.capacity() * size_of::<Pointer>()
    }
}

struct Cached {
    pointer: Pointer,
    node: Rc<ReadNode>,
}
#[derive(Default)]
pub(super) struct NodeCache {
    slots: Vec<Option<Cached>>,
    limit: usize,
    used: usize,
}
impl NodeCache {
    pub fn configure(&mut self, bytes: usize) {
        *self = Self::default();
        let slots = (bytes / size_of::<Option<Cached>>()).min(32);
        self.slots = Vec::with_capacity(slots);
        self.slots.resize_with(slots, || None);
        self.used = self.slots.capacity() * size_of::<Option<Cached>>();
        self.limit = bytes;
    }
    fn index(&self, pointer: Pointer) -> usize {
        usize::try_from(((pointer.offset >> 8) ^ (pointer.offset >> 17)) % self.slots.len() as u64)
            .unwrap()
    }
    pub fn get(&self, pointer: Pointer) -> Option<Rc<ReadNode>> {
        if self.slots.is_empty() {
            return None;
        }
        self.slots[self.index(pointer)]
            .as_ref()
            .filter(|entry| entry.pointer == pointer)
            .map(|entry| Rc::clone(&entry.node))
    }
    pub fn invalidate(&mut self, offset: u64) {
        if self.slots.is_empty() {
            return;
        }
        let index = self.index(Pointer { offset, order: 8 });
        if self.slots[index]
            .as_ref()
            .is_some_and(|entry| entry.pointer.offset == offset)
        {
            self.used -= self.slots[index].take().unwrap().node.heap_bytes();
        }
    }
    pub fn clear(&mut self) {
        self.slots.iter_mut().for_each(|slot| *slot = None);
        self.used = self.slots.capacity() * size_of::<Option<Cached>>();
    }
    pub fn put(&mut self, pointer: Pointer, node: Rc<ReadNode>) {
        if self.slots.is_empty() {
            return;
        }
        let index = self.index(pointer);
        let old_bytes = self.slots[index]
            .as_ref()
            .map_or(0, |entry| entry.node.heap_bytes());
        let used = self.used - old_bytes + node.heap_bytes();
        if used <= self.limit {
            self.slots[index] = Some(Cached { pointer, node });
            self.used = used;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_cache_budgets_include_the_slot_table() {
        let mut cache = NodeCache::default();
        for budget in 0..2048 {
            cache.configure(budget);
            assert!(cache.used <= budget, "budget={budget}, used={}", cache.used);
            let node = Rc::new(
                ReadNode::decode::<u64, u64>(Node::default().encode().unwrap(), 4096).unwrap(),
            );
            cache.put(
                Pointer {
                    offset: 4096,
                    order: 8,
                },
                node,
            );
            assert!(cache.used <= budget);
            cache.clear();
            assert!(cache.used <= budget);
        }
    }
}
