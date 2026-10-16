#![doc = include_str!("../README.md")]

//! Independent stable-memory B-trees with caller-owned key ordering.
//!
//! The tree owns its page format and allocator. [`Memory`] is the substrate
//! interface only; no upstream collection implementation is used.
//! See the package README for persistence and failure boundaries.

mod btree;

pub use btree::{BTreeMap, Comparator, Config, Entry, Error, Lexicographic, Range};
pub use ic_stable_structures::Memory;
