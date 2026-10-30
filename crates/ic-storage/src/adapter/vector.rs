//! Governed vector collection implementation.
use super::{Memory, Storable};
use crate::substrate::{self, DefaultMemoryImpl};
use ic_memory::RuntimeMemory;

/// Governed upstream `Vec`; initialization and failures follow upstream contracts.
pub struct StableVec<T: Storable, M: Memory = DefaultMemoryImpl> {
    inner: substrate::Vec<T, RuntimeMemory<M>>,
}
impl<T: Storable, M: Memory> StableVec<T, M> {
    /// Create a new collection, replacing any prior collection in the selected memory.
    /// This is destructive even when a valid collection already exists.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure.
    pub fn new(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::Vec::new(memory),
        }
    }
    /// Open a valid retained collection, or create one in zero-page memory.
    ///
    /// # Panics
    /// Panics for foreign magic, unsupported layout, incompatible element bounds,
    /// insufficient memory or codec/backing failure. Foreign memory is not reset.
    pub fn init(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::Vec::init(memory),
        }
    }
    /// Delegates to upstream `Vec::get` with the governed allocation.
    pub fn get(&self, index: u64) -> Option<T> {
        self.inner.get(index)
    }
    /// Delegates to upstream `Vec::set` with the governed allocation.
    pub fn set(&self, index: u64, item: &T) {
        self.inner.set(index, item);
    }
    /// Delegates to upstream `Vec::push` with the governed allocation.
    pub fn push(&self, item: &T) {
        self.inner.push(item);
    }
    /// Delegates to upstream `Vec::pop` with the governed allocation.
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }
    /// Reset to an empty vector while retaining this allocation's exclusive lease.
    /// Existing payload bytes follow upstream `Vec::new` semantics.
    #[must_use]
    pub fn reset(self) -> Self {
        Self::new(self.inner.into_memory())
    }
    /// Delegates to upstream `Vec::len` with the governed allocation.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Delegates to upstream `Vec::is_empty` with the governed allocation.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Delegates to upstream `Vec::iter` with the governed allocation.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = T> + '_ {
        self.inner.iter()
    }
}
