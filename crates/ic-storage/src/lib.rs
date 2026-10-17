#![doc = include_str!("../README.md")]

//! Governed stable collections consuming IC Memory runtime handles.
//! The independent page-based B-tree remains private to qualification.

pub mod adapter;

use ic_stable_structures as substrate;

#[cfg(test)]
mod experimental;
