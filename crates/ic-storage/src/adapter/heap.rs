//! Governed heap collection implementation.
use super::{Memory, Storable};
use crate::substrate::{self, DefaultMemoryImpl};
use ic_memory::RuntimeMemory;

/// Governed upstream `MinHeap`; initialization and failures follow upstream contracts.
pub struct StableMinHeap<T: Storable + Ord, M: Memory = DefaultMemoryImpl> {
    inner: substrate::MinHeap<T, RuntimeMemory<M>>,
}
impl<T: Storable + Ord, M: Memory> StableMinHeap<T, M> {
    /// Create a new collection, replacing any prior collection in the selected memory.
    /// This is destructive even when a valid collection already exists.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure.
    pub fn new(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::MinHeap::new(memory),
        }
    }
    /// Open a valid retained collection, or create one in zero-page memory.
    ///
    /// # Panics
    /// Panics for foreign magic, unsupported layout, incompatible element bounds,
    /// insufficient memory or codec/backing failure. Foreign memory is not reset.
    pub fn init(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::MinHeap::init(memory),
        }
    }
    /// Delegates to upstream `MinHeap::push` with the governed allocation.
    pub fn push(&mut self, item: &T) {
        self.inner.push(item);
    }
    /// Delegates to upstream `MinHeap::pop` with the governed allocation.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }
    /// Delegates to upstream `MinHeap::peek` with the governed allocation.
    pub fn peek(&self) -> Option<T> {
        self.inner.peek()
    }
    /// Delegates to upstream `MinHeap::len` with the governed allocation.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Delegates to upstream `MinHeap::is_empty` with the governed allocation.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Delegates to upstream `MinHeap::iter` with the governed allocation.
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.inner.iter()
    }
}
