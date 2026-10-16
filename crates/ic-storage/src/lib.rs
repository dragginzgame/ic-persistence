#![doc = include_str!("../README.md")]

//! Independent stable-memory B-trees with caller-owned key ordering.
//!
//! Collections require allocation-governed `ic-memory` runtime handles.
//! The private engine remains generic for substrate and fault-injection tests.
//! See the package README for persistence and failure boundaries.

pub mod adapter;
mod btree;

use ic_memory::RuntimeMemory;
use ic_stable_structures as substrate;
use substrate::{DefaultMemoryImpl, Memory};

pub use btree::{Comparator, Config, Error, Lexicographic};

mod governed;
pub use governed::BTreeMap;
/// A lazy entry borrowed from a governed tree.
pub type Entry<'a, M = DefaultMemoryImpl, C = Lexicographic> =
    btree::Entry<'a, RuntimeMemory<M>, C>;
/// A bidirectional range borrowed from a governed tree.
pub type Range<'a, M = DefaultMemoryImpl, C = Lexicographic> =
    btree::Range<'a, RuntimeMemory<M>, C>;
