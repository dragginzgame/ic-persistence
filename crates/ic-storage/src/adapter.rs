//! Governed collections with an explicit API.
//!
//! Storage owns every collection engine and its current format. Constructors
//! enforce explicit creation and loading without exposing raw backing.
//! Use distinct admitted allocations for owning collections. Region views retain
//! a shared lease for disjoint application frames and reload roots per operation.
//! Drop collections and all shared lease owners before reopening.
//! Collection IO and codecs return typed errors for validation and capacity
//! refusal. Backing panics/traps remain host failures; native writes are not atomic.
//! Map and set iterators yield `Result` entries and stop after an iteration error.
//! The allocation runtime enforces one live lease per allocation;
//! hosts retain exclusive custody of the backing across runtimes.
// Preserve the collection layer's pre-consolidation lint policy.
#![allow(clippy::nursery)]
#![warn(clippy::all, clippy::pedantic, missing_docs)]
use crate::DefaultMemoryImpl;
use crate::RuntimeMemory;
pub use crate::storable::{Bound, CodecError, Storable};
/// Storage-owned interfaces for backing bounds and record codecs.
/// Implementing `Memory` grants no allocation authority; constructors require
/// committed `RuntimeMemory` leases.
pub use crate::{Memory, storable};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

mod error;
mod record;
pub use error::CollectionError;
mod tree;

mod region;
pub use region::{BTreeMapRegion, RegionGrowError};

mod map;
pub use map::StableBTreeMap;
mod set;
pub use set::StableBTreeSet;
mod cell;
pub use cell::StableCell;
mod vector;
pub use vector::StableVec;

/// Ordered map iterator retaining the collection borrow.
pub type MapIter<'a, K, V, M = DefaultMemoryImpl> = tree::TreeIter<'a, K, V, RuntimeMemory<M>>;
pub use tree::MapEntry;
/// Ordered set iterator retaining the collection borrow.
pub type SetIter<'a, K, M = DefaultMemoryImpl> = set::SetIter<'a, K, M>;
