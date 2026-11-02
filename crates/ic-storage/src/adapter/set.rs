//! Governed set collection implementation.
use super::error::{CollectionError as Error, Result, ensure};
use super::{MapIter, Memory, StableBTreeMap, Storable};
use crate::DefaultMemoryImpl;
use crate::RuntimeMemory;

/// Governed ordered set sharing Storage’s owned map engine with unit values.
pub struct StableBTreeSet<K: Storable + Ord + Clone, M: Memory = DefaultMemoryImpl> {
    inner: StableBTreeMap<K, (), M>,
}
impl<K: Storable + Ord + Clone, M: Memory> StableBTreeSet<K, M> {
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
            inner: StableBTreeMap::reset(memory)?,
        })
    }

    /// Load retained collection state without initializing a replacement.
    ///
    /// # Errors
    /// Refuses empty, foreign or corrupt memory and incompatible bounds without writing.
    pub fn load(memory: RuntimeMemory<M>) -> Result<Self> {
        Ok(Self {
            inner: StableBTreeMap::load(memory)?,
        })
    }
    /// Perform `insert` on the governed ordered set.
    ///
    /// # Errors
    /// Rejects encoding, decoding, structural or capacity errors before committing writes.
    pub fn insert(&mut self, key: K) -> Result<bool> {
        Ok(self.inner.insert(key, ())?.is_none())
    }
    /// Perform `contains` on the governed ordered set.
    ///
    /// # Errors
    /// Reports structural, integrity or application decoding failures without writing.
    pub fn contains(&self, key: &K) -> Result<bool> {
        self.inner.contains_key(key)
    }
    /// Perform `is_empty` on the governed ordered set.
    ///
    /// # Errors
    /// Reports invalid region metadata or incompatible codec bounds.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Perform `len` on the governed ordered set.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Perform `remove` on the governed ordered set.
    ///
    /// # Errors
    /// Rejects decoding or structural errors before committing removal.
    pub fn remove(&mut self, key: &K) -> Result<bool> {
        Ok(self.inner.remove(key)?.is_some())
    }
    /// Perform `iter` on the governed ordered set.
    pub fn iter(&self) -> SetIter<'_, K, M> {
        SetIter(self.inner.iter())
    }
}

impl<'a, K: Storable + Ord + Clone, M: Memory> IntoIterator for &'a StableBTreeSet<K, M> {
    type Item = Result<K>;
    type IntoIter = SetIter<'a, K, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

///
/// `SetIter`
///
/// Lazy bidirectional key traversal over the shared ordered-tree engine.
///
pub struct SetIter<'a, K: Storable + Ord + Clone, M: Memory>(MapIter<'a, K, (), M>);
impl<K: Storable + Ord + Clone, M: Memory> Iterator for SetIter<'_, K, M> {
    type Item = Result<K>;
    fn next(&mut self) -> Option<Result<K>> {
        self.0
            .next()
            .map(|entry| entry.map(super::MapEntry::into_key))
    }
}
impl<K: Storable + Ord + Clone, M: Memory> DoubleEndedIterator for SetIter<'_, K, M> {
    fn next_back(&mut self) -> Option<Result<K>> {
        self.0
            .next_back()
            .map(|entry| entry.map(super::MapEntry::into_key))
    }
}
