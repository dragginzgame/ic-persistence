//! Matched map workload and bounded benchmark protocol; not a production API.
use serde::{Deserialize, Serialize};
mod types;
pub use types::*;

pub const SETUP_ID: u64 = 1;
pub const RUN_ID: u64 = 2;
pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 8 * 1024 * 1024,
    allocation: 32 * 1024 * 1024,
    nodes: 1_000_000,
    depth: 16,
};
///
/// Setup
///
/// Bounded setup or final verification request.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Setup {
    pub start: u64,
    pub count: u32,
    pub payload: u32,
}
///
/// Operation
///
/// One measured map operation, or the loop control.
///

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Control,
    Get,
    Miss,
    Replace,
    Scan,
    Insert,
    Remove,
}
pub const OPERATIONS: [Operation; 7] = [
    Operation::Control,
    Operation::Get,
    Operation::Miss,
    Operation::Replace,
    Operation::Scan,
    Operation::Insert,
    Operation::Remove,
];
///
/// Request
///
/// A deterministic operation batch shared by both backends.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Request {
    pub case: Case,
    pub operation: Operation,
    pub rows: u32,
    pub payload: u32,
    pub count: u32,
    pub seed: u64,
    pub hot: bool,
}
///
/// Response
///
/// Measured instructions and complete results for independent validation.
///

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub instructions: u64,
    pub entries: Vec<(u64, Option<Vec<u8>>)>,
    pub length: u64,
    pub stable_pages: u64,
}
/// Identical allocation/codec bytes for both engines; no Candid in the workload.
pub fn value(key: u64, length: u32, replacement: bool) -> Vec<u8> {
    (0..length)
        .map(|i| (key as u8).wrapping_add(i as u8) ^ if replacement { 128 } else { 0 })
        .collect()
}
/// Generate the complete trace outside the measured interval.
pub fn keys(request: Request) -> Vec<u64> {
    assert!((1..=16_384).contains(&request.rows));
    assert!((1..=32).contains(&request.count));
    assert!(request.payload <= 65_536);
    let mut state = request.seed;
    (0..request.count)
        .map(|index| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            match request.operation {
                Operation::Miss | Operation::Insert => u64::from(request.rows) + u64::from(index),
                _ => {
                    (state >> 32)
                        % u64::from(if request.hot {
                            request.rows.min(16)
                        } else {
                            request.rows
                        })
                }
            }
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traces_are_bounded_repeatable_and_missing_keys_are_disjoint() {
        for hot in [false, true] {
            for operation in OPERATIONS {
                let r = Request {
                    case: Case::U64Bytes,
                    operation,
                    rows: 101,
                    payload: 1024,
                    count: 32,
                    seed: 7,
                    hot,
                };
                let a = keys(r);
                assert_eq!(a, keys(r));
                assert_eq!(a.len(), 32);
                assert!(a.iter().all(|&key| {
                    if matches!(operation, Operation::Miss | Operation::Insert) {
                        key >= 101
                    } else {
                        key < if hot { 16 } else { 101 }
                    }
                }));
                let bytes = ic_wire::native::encode_request(RUN_ID, &r, LIMITS).unwrap();
                let decoded: Request =
                    ic_wire::native::decode_request(RUN_ID, &bytes, LIMITS).unwrap();
                assert_eq!(keys(decoded), a);
            }
        }
        assert_ne!(value(7, 32, false), value(7, 32, true));
    }
}
