//! Governed set collection implementation.
use super::{Memory, SetIter, Storable};
use crate::substrate::{self, DefaultMemoryImpl};
use ic_memory::RuntimeMemory;
use std::ops::RangeBounds;

/// Governed upstream `BTreeSet`; initialization and failures follow upstream contracts.
pub struct StableBTreeSet<K: Storable + Ord + Clone, M: Memory = DefaultMemoryImpl> {
    inner: substrate::BTreeSet<K, RuntimeMemory<M>>,
}
impl<K: Storable + Ord + Clone, M: Memory> StableBTreeSet<K, M> {
    /// Create a new collection, replacing any prior collection in the selected memory.
    /// This is destructive even when a valid collection already exists.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure.
    pub fn new(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::BTreeSet::new(memory),
        }
    }
    /// Open a recognized retained collection; initialize a new one when memory
    /// is empty or has foreign magic. Foreign contents may be overwritten.
    /// This is not a corruption-recovery or schema-admission operation.
    ///
    /// # Panics
    /// Panics for unsupported recognized layouts, invalid retained contents,
    /// insufficient memory or codec/backing failure.
    pub fn init(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::BTreeSet::init(memory),
        }
    }
    /// Load an existing tree without initializing a replacement.
    ///
    /// # Panics
    /// Panics for missing/foreign headers, unsupported layouts or backing failure.
    /// Record decoding can also panic when records are subsequently accessed.
    pub fn load(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::BTreeSet::load(memory),
        }
    }
    /// Delegates to upstream `BTreeSet::insert` with the governed allocation.
    pub fn insert(&mut self, key: K) -> bool {
        self.inner.insert(key)
    }
    /// Delegates to upstream `BTreeSet::contains` with the governed allocation.
    pub fn contains(&self, key: &K) -> bool {
        self.inner.contains(key)
    }
    /// Delegates to upstream `BTreeSet::is_empty` with the governed allocation.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Delegates to upstream `BTreeSet::len` with the governed allocation.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Delegates to upstream `BTreeSet::remove` with the governed allocation.
    pub fn remove(&mut self, key: &K) -> bool {
        self.inner.remove(key)
    }
    /// Delegates to upstream `BTreeSet::first` with the governed allocation.
    pub fn first(&self) -> Option<K> {
        self.inner.first()
    }
    /// Delegates to upstream `BTreeSet::last` with the governed allocation.
    pub fn last(&self) -> Option<K> {
        self.inner.last()
    }
    /// Delegates to upstream `BTreeSet::pop_first` with the governed allocation.
    pub fn pop_first(&mut self) -> Option<K> {
        self.inner.pop_first()
    }
    /// Delegates to upstream `BTreeSet::pop_last` with the governed allocation.
    pub fn pop_last(&mut self) -> Option<K> {
        self.inner.pop_last()
    }
    /// Delegates to upstream `BTreeSet::iter` with the governed allocation.
    pub fn iter(&self) -> SetIter<'_, K, M> {
        self.inner.iter()
    }
    /// Delegates to upstream `BTreeSet::range` with the governed allocation.
    pub fn range(&self, key_range: impl RangeBounds<K>) -> SetIter<'_, K, M> {
        self.inner.range(key_range)
    }
    /// Delegates to upstream `BTreeSet::clear` with the governed allocation.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<'a, K: Storable + Ord + Clone, M: Memory> IntoIterator for &'a StableBTreeSet<K, M> {
    type Item = K;
    type IntoIter = SetIter<'a, K, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
