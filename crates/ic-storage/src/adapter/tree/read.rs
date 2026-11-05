//! Checked read views retain one record buffer. The optional cache is heap-only.
use super::{
    Entry, Error, INLINE, MAX_ENTRIES, NODE, Node, Pointer, Reader, Result, Storable, Value,
    ensure, record,
};
use std::{borrow::Cow, cmp::Ordering, ops::Range, rc::Rc};

// Keep these small instrumentation declarations beside the methods they observe;
// test bodies and ordinary fixture helpers live in read/tests.rs.
#[cfg(test)]
thread_local! { pub(super) static OWNED_NODES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

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
    pub const fn len(&self) -> usize {
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
    pub fn inline_value(&self, index: usize) -> Option<&[u8]> {
        match &self.entries[index].value {
            ReadValue::Inline(range) => Some(&self.bytes[range.clone()]),
            ReadValue::External(_) => None,
        }
    }
    pub fn key_bytes(&self, index: usize) -> &[u8] {
        &self.bytes[self.entries[index].key.clone()]
    }
    // Replace only an admitted inline span. Copy unchanged encodings into one
    // final frame instead of materializing every key/value, and seal it through
    // the same record writer as ordinary node encoding.
    pub fn replace_inline(&self, index: usize, value: &[u8]) -> Result<Vec<u8>> {
        ensure(value.len() <= INLINE, Error::Budget)?;
        let ReadValue::Inline(range) = &self.entries[index].value else {
            return Err(Error::Corrupt("expected inline replacement"));
        };
        let length = self.bytes.len() - record::PREFIX - range.len() + value.len();
        record::encode_payload_with(NODE, length, |bytes| {
            bytes.extend_from_slice(&self.bytes[record::PREFIX..range.start - 1]);
            bytes.push(u8::try_from(value.len()).unwrap());
            bytes.extend_from_slice(value);
            bytes.extend_from_slice(&self.bytes[range.end..]);
            Ok(())
        })
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
        #[cfg(test)]
        OWNED_NODES.with(|count| count.set(count.get() + 1));
        Node {
            entries: (0..self.len()).map(|i| self.entry(i)).collect(),
            children: self.children.clone(),
        }
    }
    const fn heap_bytes(&self) -> usize {
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
    depth: usize,
}
#[cfg(test)]
#[derive(Default, Debug)]
pub(super) struct CacheMetrics {
    pub hits: u64,
    pub cold_misses: u64,
    pub collision_misses: u64,
    pub protected_rejections: u64,
    pub budget_rejections: u64,
}
#[derive(Default)]
pub(super) struct NodeCache {
    slots: Vec<Option<Cached>>,
    limit: usize,
    used: usize,
    #[cfg(test)]
    pub metrics: CacheMetrics,
}
impl NodeCache {
    pub fn configure(&mut self, bytes: usize) {
        *self = Self::default();
        // Two slots per set, with the same approximate table/payload split.
        let slots = bytes / (1024 + size_of::<Option<Cached>>()) / 2 * 2;
        self.slots = Vec::with_capacity(slots);
        self.slots.resize_with(slots, || None);
        self.used = self.slots.capacity() * size_of::<Option<Cached>>();
        self.limit = bytes;
    }
    fn index(&self, pointer: Pointer) -> usize {
        usize::try_from(
            ((pointer.offset >> 8) ^ (pointer.offset >> 17)) % (self.slots.len() / 2) as u64,
        )
        .unwrap()
            * 2
    }
    pub fn get(&mut self, pointer: Pointer, depth: usize) -> Option<Rc<ReadNode>> {
        if self.slots.is_empty() {
            return None;
        }
        let start = self.index(pointer);
        for index in start..start + 2 {
            if let Some(entry) = &mut self.slots[index]
                && entry.pointer == pointer
            {
                entry.depth = depth;
                let node = Rc::clone(&entry.node);
                // First way is most recently used; no timestamps or extra table.
                if index != start {
                    self.slots.swap(start, index);
                }
                #[cfg(test)]
                {
                    self.metrics.hits += 1;
                }
                return Some(node);
            }
        }
        #[cfg(test)]
        {
            if self.slots[start..start + 2].iter().all(Option::is_some) {
                self.metrics.collision_misses += 1;
            } else {
                self.metrics.cold_misses += 1;
            }
        }
        None
    }
    pub fn invalidate(&mut self, offset: u64) {
        if self.slots.is_empty() {
            return;
        }
        let start = self.index(Pointer { offset, order: 8 });
        for index in start..start + 2 {
            if self.slots[index]
                .as_ref()
                .is_some_and(|entry| entry.pointer.offset == offset)
            {
                self.used -= self.slots[index].take().unwrap().node.heap_bytes();
            }
        }
    }
    pub fn clear(&mut self) {
        self.slots.iter_mut().for_each(|slot| *slot = None);
        self.used = self.slots.capacity() * size_of::<Option<Cached>>();
    }
    pub fn put(&mut self, pointer: Pointer, node: Rc<ReadNode>, depth: usize) {
        if self.slots.is_empty() {
            return;
        }
        let start = self.index(pointer);
        let eligible = |index: usize| {
            !self.slots[index]
                .as_ref()
                .is_some_and(|old| old.pointer != pointer && old.depth <= 1 && depth > old.depth)
        };
        // Update in place, use an empty way, then evict the least-recent eligible way.
        let index = (start..start + 2)
            .find(|&i| {
                self.slots[i]
                    .as_ref()
                    .is_some_and(|old| old.pointer == pointer)
            })
            .or_else(|| (start..start + 2).find(|&i| self.slots[i].is_none()))
            .or_else(|| (start..start + 2).rev().find(|&i| eligible(i)));
        let Some(index) = index else {
            #[cfg(test)]
            {
                self.metrics.protected_rejections += 1;
            }
            return;
        };
        let old_bytes = self.slots[index]
            .as_ref()
            .map_or(0, |entry| entry.node.heap_bytes());
        let used = self.used - old_bytes + node.heap_bytes();
        if used <= self.limit {
            self.slots[index] = Some(Cached {
                pointer,
                node,
                depth,
            });
            self.used = used;
            if index != start {
                self.slots.swap(start, index);
            }
        } else {
            #[cfg(test)]
            {
                self.metrics.budget_rejections += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests;
