//! Private custom-tree qualification surface, also compiled by the unpublished benchmark.
use ic_memory::{DefaultMemoryImpl, Memory, RuntimeMemory};

#[path = "btree/mod.rs"]
mod btree;
#[path = "governed.rs"]
mod governed;

pub(crate) use btree::{Comparator, Config, Error, Lexicographic};
pub(crate) use governed::BTreeMap;
pub(crate) type Entry<'a, M = DefaultMemoryImpl, C = Lexicographic> =
    btree::Entry<'a, RuntimeMemory<M>, C>;
pub(crate) type Range<'a, M = DefaultMemoryImpl, C = Lexicographic> =
    btree::Range<'a, RuntimeMemory<M>, C>;
