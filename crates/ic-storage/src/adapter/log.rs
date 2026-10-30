//! Governed log collection implementation.
use super::{LogIter, LogWriteError, Memory, Storable};
use crate::substrate::{self, DefaultMemoryImpl};
use ic_memory::RuntimeMemory;

/// Governed upstream `Log`; initialization and failures follow upstream contracts.
pub struct StableLog<T: Storable, M: Memory = DefaultMemoryImpl> {
    inner: substrate::Log<T, RuntimeMemory<M>, RuntimeMemory<M>>,
}
impl<T: Storable, M: Memory> StableLog<T, M> {
    /// Create a new collection, replacing any prior collection in the selected memory.
    /// This is destructive even when a valid collection already exists.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure.
    pub fn new(index_memory: RuntimeMemory<M>, data_memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::Log::new(index_memory, data_memory),
        }
    }
    /// Open a retained log, or initialize both memories when data memory is
    /// empty or has foreign magic. The latter overwrites previous contents.
    ///
    /// # Panics
    /// With recognized data magic, panics for an invalid index or unsupported
    /// layout. Growth and codec/backing failures also retain upstream panics.
    pub fn init(index_memory: RuntimeMemory<M>, data_memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::Log::init(index_memory, data_memory),
        }
    }
    /// Delegates to upstream `Log::first` with the governed allocation.
    pub fn first(&self) -> Option<T> {
        self.inner.first()
    }
    /// Delegates to upstream `Log::last` with the governed allocation.
    pub fn last(&self) -> Option<T> {
        self.inner.last()
    }
    /// Delegates to upstream `Log::get` with the governed allocation.
    pub fn get(&self, idx: u64) -> Option<T> {
        self.inner.get(idx)
    }
    /// Delegates to upstream `Log::len` with the governed allocation.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Delegates to upstream `Log::is_empty` with the governed allocation.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Delegates to upstream `Log::append` with the governed allocation.
    /// # Errors
    /// Returns the upstream write error if memory cannot grow.
    pub fn append(&self, item: &T) -> Result<u64, LogWriteError> {
        self.inner.append(item)
    }
    /// Delegates to upstream `Log::iter` with the governed allocation.
    pub fn iter(&self) -> LogIter<'_, T, M> {
        self.inner.iter()
    }
}

impl<'a, T: Storable, M: Memory> IntoIterator for &'a StableLog<T, M> {
    type Item = T;
    type IntoIter = LogIter<'a, T, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
