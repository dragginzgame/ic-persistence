//! Supported upstream collections with governed memory and an explicit API.
//!
//! Constructors preserve upstream initialization, panic and persisted-format
//! contracts. These wrappers expose neither raw memory nor unrestricted
//! upstream collections.
//! Use distinct admitted allocations for owning collections. Region views retain
//! a shared lease for disjoint application frames and reload roots per operation.
//! Drop collections and all shared lease owners before reopening.
//! IC Memory enforces one live lease per allocation within each runtime;
//! hosts retain exclusive custody of the backing across runtimes.
use crate::substrate::{self, DefaultMemoryImpl};
use ic_memory::RuntimeMemory;
use std::{borrow::Cow, ops::RangeBounds};
/// Exact substrate interfaces for backing bounds and record codecs.
/// Implementing `Memory` grants no allocation authority; constructors require
/// committed `RuntimeMemory` leases.
pub use substrate::{Memory, Storable, storable, storable::Bound};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

mod region;
pub use region::{BTreeMapRegion, RegionGrowError};

/// Governed upstream `BTreeMap`; initialization and failures follow upstream contracts.
pub struct StableBTreeMap<K: Storable + Ord + Clone, V: Storable, M: Memory = DefaultMemoryImpl> {
    inner: substrate::BTreeMap<K, V, RuntimeMemory<M>>,
}
impl<K: Storable + Ord + Clone, V: Storable, M: Memory> StableBTreeMap<K, V, M> {
    /// Create a new collection, replacing any prior collection in the selected memory.
    /// This is destructive even when a valid collection already exists.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure.
    pub fn new(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::BTreeMap::new(memory),
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
            inner: substrate::BTreeMap::init(memory),
        }
    }
    /// Load an existing tree without initializing a replacement.
    ///
    /// # Panics
    /// Panics for missing/foreign headers, unsupported layouts or backing failure.
    /// Record decoding can also panic when records are subsequently accessed.
    pub fn load(memory: RuntimeMemory<M>) -> Self {
        Self {
            inner: substrate::BTreeMap::load(memory),
        }
    }
    /// Clear entries while retaining this allocation's exclusive lease.
    pub fn clear(&mut self) {
        self.inner.clear_new();
    }
    /// Reserve a minimum virtual extent without exposing or duplicating the lease.
    ///
    /// Returns the map on both success and ordinary growth refusal. Entries are
    /// preserved; reloading the upstream header refreshes its cached allocator.
    /// Backing traps retain the upstream panic contract.
    ///
    /// `minimum` is an absolute virtual extent in 65,536-byte Wasm pages, not an
    /// entry count or additional pages. Existing extents never shrink. A request
    /// already covered performs no growth or writes. This reserves backing only:
    /// it does not promise that any particular number or encoding of entries fits.
    /// Callers own their byte/page budget and must qualify it for their codecs;
    /// upstream node-layout constants are not a supported capacity contract.
    pub fn reserve_pages(self, minimum: u64) -> (Self, Result<(), ic_memory::RuntimeGrowError>) {
        let memory = self.inner.into_memory();
        let additional = minimum.saturating_sub(memory.size());
        let result = memory.grow(additional).map(|_| ());
        (Self::load(memory), result)
    }
    /// Observe the virtual extent in 65,536-byte pages while retaining the lease.
    ///
    /// Consumes and returns the map because upstream exposes memory only through
    /// ownership transfer. Reloading reads the header without writing or growing.
    /// This is allocated extent, not free space or guaranteed entry capacity.
    #[must_use]
    pub fn allocated_pages(self) -> (Self, u64) {
        let memory = self.inner.into_memory();
        let pages = memory.size();
        (Self::load(memory), pages)
    }
    /// Delegates to upstream `BTreeMap::insert` with the governed allocation.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }
    /// Delegates to upstream `BTreeMap::get` with the governed allocation.
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key)
    }
    /// Delegates to upstream `BTreeMap::contains_key` with the governed allocation.
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }
    /// Delegates to upstream `BTreeMap::is_empty` with the governed allocation.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Delegates to upstream `BTreeMap::len` with the governed allocation.
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    /// Delegates to upstream `BTreeMap::remove` with the governed allocation.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }
    /// Delegates to upstream `BTreeMap::first_key_value` with the governed allocation.
    pub fn first_key_value(&self) -> Option<(K, V)> {
        self.inner.first_key_value()
    }
    /// Delegates to upstream `BTreeMap::last_key_value` with the governed allocation.
    pub fn last_key_value(&self) -> Option<(K, V)> {
        self.inner.last_key_value()
    }
    /// Delegates to upstream `BTreeMap::pop_first` with the governed allocation.
    pub fn pop_first(&mut self) -> Option<(K, V)> {
        self.inner.pop_first()
    }
    /// Delegates to upstream `BTreeMap::pop_last` with the governed allocation.
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        self.inner.pop_last()
    }
    /// Delegates to upstream `BTreeMap::iter` with the governed allocation.
    pub fn iter(&self) -> MapIter<'_, K, V, M> {
        self.inner.iter()
    }
    /// Delegates to upstream `BTreeMap::range` with the governed allocation.
    pub fn range(&self, key_range: impl RangeBounds<K>) -> MapIter<'_, K, V, M> {
        self.inner.range(key_range)
    }
    /// Delegates to upstream `BTreeMap::keys` with the governed allocation.
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = K> + '_ {
        self.inner.keys()
    }
    /// Delegates to upstream `BTreeMap::values` with the governed allocation.
    pub fn values(&self) -> impl DoubleEndedIterator<Item = V> + '_ {
        self.inner.values()
    }
}

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

///
/// `StableCell`
///
/// Governed upstream `Cell` with explicit initialization and read-only loading.
///
/// Values and stored bytes retain the upstream codec and layout contracts.
///
pub struct StableCell<T: Storable, M: Memory = DefaultMemoryImpl> {
    inner: substrate::Cell<CellValue<T>, RuntimeMemory<M>>,
}

// Upstream has no load constructor. An absent value is an internal loading
// marker, never a default T or a persisted record. Serialization refuses it
// before upstream can grow/write on its initialization fallback. Successful
// construction always holds Some(T), decoded exactly once by upstream.
struct CellValue<T>(Option<T>);

impl<T: Storable> Storable for CellValue<T> {
    const BOUND: Bound = T::BOUND;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.0
            .as_ref()
            .expect("cannot initialize a loading cell")
            .to_bytes()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.0
            .expect("cannot initialize a loading cell")
            .into_bytes()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(Some(T::from_bytes(bytes)))
    }
}

impl<T: Storable, M: Memory> StableCell<T, M> {
    /// Create a new collection, replacing any prior collection in the selected memory.
    /// This is destructive even when a valid collection already exists.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure.
    pub fn new(memory: RuntimeMemory<M>, value: T) -> Self {
        Self {
            inner: substrate::Cell::new(memory, CellValue(Some(value))),
        }
    }
    /// Open a recognized retained collection; initialize a new one when memory
    /// is empty or has foreign magic. Foreign contents may be overwritten.
    /// This is not a corruption-recovery or schema-admission operation.
    ///
    /// # Panics
    /// Panics for unsupported recognized layouts, invalid retained contents,
    /// insufficient memory or codec/backing failure.
    pub fn init(memory: RuntimeMemory<M>, default_value: T) -> Self {
        Self {
            inner: substrate::Cell::init(memory, CellValue(Some(default_value))),
        }
    }
    /// Load an existing cell without initializing, writing or growing memory.
    ///
    /// Requires no default value or `Default`/`Clone` bound. The retained value
    /// is decoded by upstream using `T::from_bytes`. A successful load retains
    /// the allocation's exclusive lease until this cell is dropped.
    ///
    /// # Errors
    /// Returns [`ic_memory::StableCellPayloadError`] for empty/foreign memory,
    /// unsupported layout versions or a payload extending beyond memory. The
    /// consumed lease is released on failure; no bytes are changed.
    ///
    /// # Panics
    /// Codec and backing failures propagate as upstream panics. Loading does
    /// not write or grow even on these failures. Ordinary unwinding releases
    /// the consumed lease; Wasm traps retain the host's rollback semantics.
    /// This validates the envelope, not the application's schema or recovery policy.
    pub fn load(memory: RuntimeMemory<M>) -> Result<Self, ic_memory::StableCellPayloadError> {
        ic_memory::stable_cell_payload_length(&memory, usize::MAX)?;
        Ok(Self {
            inner: substrate::Cell::init(memory, CellValue(None)),
        })
    }
    /// Delegates to upstream `Cell::get` with the governed allocation.
    #[expect(
        clippy::missing_panics_doc,
        reason = "constructors always establish a cached value"
    )]
    pub fn get(&self) -> &T {
        self.inner.get().0.as_ref().expect("loaded cell value")
    }
    /// Delegates to upstream `Cell::set` with the governed allocation.
    ///
    /// # Panics
    /// Panics on insufficient memory or codec/backing failure, as upstream.
    pub fn set(&mut self, value: T) -> T {
        self.inner
            .set(CellValue(Some(value)))
            .0
            .expect("loaded cell value")
    }
}

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

/// Upstream map iterator with governed memory.
pub type MapIter<'a, K, V, M = DefaultMemoryImpl> =
    substrate::btreemap::Iter<'a, K, V, RuntimeMemory<M>>;
/// Upstream set iterator with governed memory.
pub type SetIter<'a, K, M = DefaultMemoryImpl> = substrate::btreeset::Iter<'a, K, RuntimeMemory<M>>;
/// Upstream log iterator with governed index and data memories.
pub type LogIter<'a, T, M = DefaultMemoryImpl> =
    substrate::log::Iter<'a, T, RuntimeMemory<M>, RuntimeMemory<M>>;

pub use substrate::log::WriteError as LogWriteError;

impl<'a, K: Storable + Ord + Clone, V: Storable, M: Memory> IntoIterator
    for &'a StableBTreeMap<K, V, M>
{
    type Item = <MapIter<'a, K, V, M> as Iterator>::Item;
    type IntoIter = MapIter<'a, K, V, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Storable + Ord + Clone, M: Memory> IntoIterator for &'a StableBTreeSet<K, M> {
    type Item = K;
    type IntoIter = SetIter<'a, K, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Storable, M: Memory> IntoIterator for &'a StableLog<T, M> {
    type Item = T;
    type IntoIter = LogIter<'a, T, M>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
/// A stable cell whose writes enforce the record's declared `Storable` bound.
pub struct BoundedCell<T: Storable, M: Memory = DefaultMemoryImpl> {
    cell: substrate::Cell<BoundedValue<T>, RuntimeMemory<M>>,
}

impl<T: Storable, M: Memory> BoundedCell<T, M> {
    /// Open a current cell or initialize a fresh bounded record.
    /// Empty memory or foreign magic initializes the supplied value, potentially
    /// overwriting foreign contents. Recognized incompatible layouts are refused.
    ///
    /// # Panics
    /// Panics for unbounded records or encodings that violate the declared bound.
    /// Also retains [`StableCell::init`] layout, growth and codec/backing panics.
    #[must_use]
    pub fn init(memory: RuntimeMemory<M>, value: T) -> Self {
        assert!(matches!(T::BOUND, Bound::Bounded { .. }));
        Self {
            cell: substrate::Cell::init(memory, BoundedValue(value)),
        }
    }

    /// Read the cached current record without serializing or cloning it.
    #[must_use]
    pub fn get(&self) -> &T {
        &self.cell.get().0
    }

    /// Validate and persist a complete replacement, returning the prior record.
    ///
    /// # Panics
    /// Panics before writing if the encoding violates its declared bound.
    pub fn set(&mut self, value: T) -> T {
        self.cell.set(BoundedValue(value)).0
    }
}

struct BoundedValue<T>(T);

impl<T: Storable> Storable for BoundedValue<T> {
    const BOUND: Bound = T::BOUND;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.0.to_bytes_checked()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes_checked()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        if matches!(T::BOUND, Bound::Unbounded) {
            unreachable!("bounded cell record must declare a bound");
        }
        T::check_bounds(&bytes);
        Self(T::from_bytes(bytes))
    }
}
