//! Governed cell collection implementation.
use super::{Bound, Memory, Storable};
use crate::substrate::{self, DefaultMemoryImpl};
use ic_memory::RuntimeMemory;
use std::borrow::Cow;

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
