//! Governed map collection implementation.
use super::error::{CollectionError as Error, Result, ensure};
use super::tree::Tree;
use super::{MapIter, Memory, Storable};
use crate::DefaultMemoryImpl;
use crate::RuntimeMemory;
use std::ops::RangeBounds;

/// Governed ordered map backed by Storage’s owned AVL tree and reusable blocks.
pub struct StableBTreeMap<K: Storable + Ord + Clone, V: Storable, M: Memory = DefaultMemoryImpl> {
    inner: Tree<K, V, RuntimeMemory<M>>,
}
impl<K: Storable + Ord + Clone, V: Storable, M: Memory> StableBTreeMap<K, V, M> {
    /// Create a collection only in zero-page memory.
    ///
    /// # Errors
    /// Refuses nonempty memory, invalid encoding or capacity exhaustion before writes.
    pub fn create(memory: RuntimeMemory<M>) -> Result<Self> {
        ensure(memory.size() == 0, Error::NotEmpty)?;
        Self::reset(memory)
    }

    /// Deliberately replace any contents in this admitted allocation.
    ///
    /// # Errors
    /// Returns encoding, capacity or structural admission errors.
    pub fn reset(memory: RuntimeMemory<M>) -> Result<Self> {
        Ok(Self {
            inner: Tree::reset(memory)?,
        })
    }

    /// Load retained collection state without initializing a replacement.
    ///
    /// # Errors
    /// Refuses empty, foreign or corrupt memory and incompatible bounds without writing.
    pub fn load(memory: RuntimeMemory<M>) -> Result<Self> {
        Ok(Self {
            inner: Tree::load(memory)?,
        })
    }
    /// Clear entries while retaining this allocation's exclusive lease.
    ///
    /// # Errors
    /// Reports a poisoned owner or metadata encoding/capacity refusal.
    pub fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    /// Reserve a minimum virtual extent without exposing or duplicating the lease.
    ///
    /// Returns the map on both success and ordinary growth refusal. Entries are
    /// preserved; the cached tree remains unchanged.
    /// Backing traps follow the host transaction contract.
    ///
    /// `minimum` is an absolute virtual extent in 65,536-byte Wasm pages, not an
    /// entry count or additional pages. Existing extents never shrink. A request
    /// already covered performs no growth or writes. This reserves backing only:
    /// it does not promise that any particular number or encoding of entries fits.
    /// Callers own their byte/page budget and must qualify it for their codecs;
    /// internal node-layout constants are not a supported capacity contract.
    pub fn reserve_pages(
        self,
        minimum: u64,
    ) -> (Self, std::result::Result<(), crate::RuntimeGrowError>) {
        let additional = minimum.saturating_sub(self.inner.memory.size());
        let result = self.inner.memory.grow(additional).map(|_| ());
        (self, result)
    }
    /// Observe the virtual extent in 65,536-byte pages while retaining the lease.
    ///
    /// Consumes and returns the map while retaining the exclusive lease.
    /// Observing the allocation performs no writes or growth.
    /// This is allocated extent, not free space or guaranteed entry capacity.
    #[must_use]
    pub fn allocated_pages(self) -> (Self, u64) {
        let pages = self.inner.memory.size();
        (self, pages)
    }
    /// Perform `insert` on the governed ordered map.
    ///
    /// # Errors
    /// Rejects encoding, decoding, structural or capacity errors before committing writes.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        self.inner.insert(key, value)
    }
    /// Perform `get` on the governed ordered map.
    ///
    /// # Errors
    /// Reports structural, integrity or application decoding failures without writing.
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        self.inner.get(key)
    }
    /// Perform `contains_key` on the governed ordered map.
    ///
    /// # Errors
    /// Reports structural, integrity or application decoding failures without writing.
    pub fn contains_key(&self, key: &K) -> Result<bool> {
        Ok(self.inner.get(key)?.is_some())
    }
    /// Perform `is_empty` on the governed ordered map.
    ///
    /// # Errors
    /// Reports invalid region metadata or incompatible codec bounds.
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }
    /// Perform `len` on the governed ordered map.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Perform `remove` on the governed ordered map.
    ///
    /// # Errors
    /// Rejects decoding or structural errors before committing removal.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        self.inner.remove(key)
    }
    /// Perform `first_key_value` on the governed ordered map.
    ///
    /// # Errors
    /// Reports invalid structure, integrity failures or application codec errors.
    pub fn first_key_value(&self) -> Result<Option<(K, V)>> {
        self.iter()
            .next()
            .map(|entry| entry?.into_pair())
            .transpose()
    }
    /// Perform `last_key_value` on the governed ordered map.
    ///
    /// # Errors
    /// Reports invalid structure, integrity failures or application codec errors.
    pub fn last_key_value(&self) -> Result<Option<(K, V)>> {
        self.iter()
            .next_back()
            .map(|entry| entry?.into_pair())
            .transpose()
    }
    /// Perform `iter` on the governed ordered map.
    pub fn iter(&self) -> MapIter<'_, K, V, M> {
        self.inner.range(..)
    }
    /// Perform `range` on the governed ordered map.
    pub fn range(&self, key_range: impl RangeBounds<K>) -> MapIter<'_, K, V, M> {
        self.inner.range(key_range)
    }
    /// Perform `keys` on the governed ordered map.
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = Result<K>> + '_ {
        self.iter()
            .map(|entry| entry.map(super::MapEntry::into_key))
    }
    /// Perform `values` on the governed ordered map.
    pub fn values(&self) -> impl DoubleEndedIterator<Item = Result<V>> + '_ {
        self.iter().map(|entry| entry?.into_value())
    }
}

impl<'a, K: Storable + Ord + Clone, V: Storable, M: Memory> IntoIterator
    for &'a StableBTreeMap<K, V, M>
{
    type Item = <MapIter<'a, K, V, M> as Iterator>::Item;
    type IntoIter = MapIter<'a, K, V, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
