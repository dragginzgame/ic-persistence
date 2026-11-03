//! Checked read views retain one record buffer. The optional cache is heap-only.
use super::{
    Entry, Error, INLINE, MAX_ENTRIES, NODE, Node, Pointer, Reader, Result, Storable, Value,
    ensure, record,
};
use std::{borrow::Cow, cmp::Ordering, ops::Range, rc::Rc};

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
    pub fn inline_value(&self, index: usize) -> Option<&[u8]> {
        match &self.entries[index].value {
            ReadValue::Inline(range) => Some(&self.bytes[range.clone()]),
            ReadValue::External(_) => None,
        }
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
        // Reserve most of the budget for node buffers, with roughly one slot
        // per KiB. Actual buffer capacities, not this estimate, bound admission.
        let slots = bytes / (1024 + size_of::<Option<Cached>>());
        self.slots = Vec::with_capacity(slots);
        self.slots.resize_with(slots, || None);
        self.used = self.slots.capacity() * size_of::<Option<Cached>>();
        self.limit = bytes;
    }
    fn index(&self, pointer: Pointer) -> usize {
        usize::try_from(((pointer.offset >> 8) ^ (pointer.offset >> 17)) % self.slots.len() as u64)
            .unwrap()
    }
    pub fn get(&mut self, pointer: Pointer, depth: usize) -> Option<Rc<ReadNode>> {
        if self.slots.is_empty() {
            return None;
        }
        let index = self.index(pointer);
        match &mut self.slots[index] {
            Some(entry) if entry.pointer == pointer => {
                entry.depth = depth;
                #[cfg(test)]
                {
                    self.metrics.hits += 1;
                }
                Some(Rc::clone(&entry.node))
            }
            slot => {
                #[cfg(test)]
                {
                    if slot.is_some() {
                        self.metrics.collision_misses += 1;
                    } else {
                        self.metrics.cold_misses += 1;
                    }
                }
                let _ = slot;
                None
            }
        }
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
    pub fn put(&mut self, pointer: Pointer, node: Rc<ReadNode>, depth: usize) {
        if self.slots.is_empty() {
            return;
        }
        let index = self.index(pointer);
        if self.slots[index]
            .as_ref()
            .is_some_and(|old| old.pointer != pointer && old.depth <= 1 && depth > old.depth)
        {
            #[cfg(test)]
            {
                self.metrics.protected_rejections += 1;
            }
            return;
        }
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
        } else {
            #[cfg(test)]
            {
                self.metrics.budget_rejections += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
            let node = Rc::new(
                ReadNode::decode::<u64, u64>(Node::default().encode().unwrap(), 4096).unwrap(),
            );
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
}
