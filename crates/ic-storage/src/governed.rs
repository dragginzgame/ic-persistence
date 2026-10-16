//! Public custom-tree boundary. Allocation admission is supplied by IC Memory.
use crate::substrate::DefaultMemoryImpl;
use crate::{Comparator, Config, Entry, Error, Lexicographic, Memory, Range, btree};
use ic_memory::RuntimeMemory;
use std::ops::Bound;

/// Our page-based tree consuming a governed allocation handle.
/// No raw memory access or extraction is exposed; drop before reopening.
pub struct BTreeMap<M: Memory = DefaultMemoryImpl, C: Comparator = Lexicographic> {
    inner: btree::BTreeMap<RuntimeMemory<M>, C>,
}
impl<M: Memory, C: Comparator> BTreeMap<M, C> {
    /// Create the custom format with the exact configuration and comparator.
    /// # Errors
    /// Returns the engine's format, configuration, corruption or capacity error.
    pub fn create(memory: RuntimeMemory<M>, config: Config, order: C) -> Result<Self, Error> {
        btree::BTreeMap::create(memory, config, order).map(|inner| Self { inner })
    }
    /// Open the custom format with the exact configuration and comparator.
    /// # Errors
    /// Returns the engine's format, configuration, corruption or capacity error.
    pub fn open(memory: RuntimeMemory<M>, config: Config, order: C) -> Result<Self, Error> {
        btree::BTreeMap::open(memory, config, order).map(|inner| Self { inner })
    }
    /// Number of persisted live entries.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Whether there are no live entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Bytes reserved, including reusable free pages.
    #[must_use]
    pub fn memory_bytes(&self) -> u64 {
        self.inner.memory_bytes()
    }
    /// Read the value for an encoded key.
    /// # Errors
    /// Propagates validation, corruption and capacity errors from the engine.
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.inner.get(key)
    }
    /// Test membership without reading external value payloads.
    /// # Errors
    /// Propagates validation, corruption and capacity errors from the engine.
    pub fn contains_key(&self, key: &[u8]) -> Result<bool, Error> {
        self.inner.contains_key(key)
    }
    /// Insert or replace a value, returning its previous bytes.
    /// # Errors
    /// Propagates validation, corruption and capacity errors from the engine.
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.inner.insert(key, value)
    }
    /// Remove a key, returning its previous value.
    /// # Errors
    /// Propagates validation, corruption and capacity errors from the engine.
    pub fn remove(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        self.inner.remove(key)
    }
    /// Lazily scan comparator-ordered bounds in either direction.
    /// # Errors
    /// Propagates validation, corruption and capacity errors from the engine.
    pub fn range(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> Result<Range<'_, M, C>, Error> {
        self.inner.range(lower, upper)
    }
    /// Lazily scan all entries in either direction.
    #[must_use]
    pub fn iter(&self) -> Range<'_, M, C> {
        self.inner.iter()
    }
    /// Audit reachable pages, ordering, counts and free-list ownership.
    /// # Errors
    /// Propagates validation, corruption and capacity errors from the engine.
    pub fn validate(&self) -> Result<(), Error> {
        self.inner.validate()
    }
}
impl<'a, M: Memory, C: Comparator> IntoIterator for &'a BTreeMap<M, C> {
    type Item = Result<Entry<'a, M, C>, Error>;
    type IntoIter = Range<'a, M, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
