//! Shared state and protocol for the qualification examples.

pub mod host;
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)] // Only the IO qualification target consumes this protocol.
pub mod io_protocol;
