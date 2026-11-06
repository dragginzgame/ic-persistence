//! Diagnostic protocol for raw stable-read accounting over a populated vector image.
use serde::{Deserialize, Serialize};

pub const ID: u64 = 0x5352_5052;
pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 32 * 1024,
    allocation: 256 * 1024,
    nodes: 32 * 1024,
    depth: 8,
};

///
/// Probe
///
/// Physical source range found in the retained vector image by the host.
/// This is a research fixture, not a production allocation interface.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Probe {
    pub offset: u64,
    pub prefix: u32,
    pub payload: u32,
    pub combined: bool,
}

///
/// Read
///
/// One system-call measurement; buffers are prepared before its counter.
///

#[derive(Debug, Serialize, Deserialize)]
pub struct Read {
    pub phase: String,
    pub offset: u64,
    pub destination: u64,
    pub instructions: u64,
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests;
