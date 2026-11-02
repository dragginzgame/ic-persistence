//! Matched map workload and bounded benchmark protocol; not a production API.
use serde::{Deserialize, Serialize};
mod types;
#[cfg(all(
    target_arch = "wasm32",
    feature = "dfinity",
    not(feature = "dfinity-head")
))]
pub use ic_platform_stable_benchmark::stable_structures as upstream;
#[cfg(all(target_arch = "wasm32", feature = "dfinity-head"))]
pub use ic_stable_structures_head as upstream;
pub use types::*;

pub const MAX_ROWS: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::ValueEnum))]
pub enum Pattern {
    #[default]
    Uniform,
    Hot16,
    Zipf,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Initialization {
    pub case: Case,
    pub cache_bytes: usize,
    pub upstream_cache_slots: usize,
    pub cold_cache: bool,
}

pub const SETUP_ID: u64 = 1;
pub const RUN_ID: u64 = 2;
pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 8 * 1024 * 1024,
    // Wire conservatively charges visited byte references as well as buffers.
    allocation: 128 * 1024 * 1024,
    // Vec<u8> visits each byte; allow the largest 512 KiB result plus framing.
    nodes: 4_000_000,
    depth: 16,
};
///
/// Setup
///
/// Bounded setup or final verification request.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Setup {
    pub start: u64,
    pub count: u32,
    pub payload: u32,
    pub ids: Vec<u64>,
}

pub fn population_ids(setup: &Setup) -> Vec<u64> {
    if setup.ids.is_empty() {
        (setup.start..setup.start + u64::from(setup.count)).collect()
    } else {
        assert_eq!(setup.ids.len(), setup.count as usize);
        setup.ids.clone()
    }
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
    Contains,
    ReverseScan,
    KeysScan,
    Grow,
    Shrink,
    RemoveMiss,
    Reinsert,
    Set,
    Push,
    Pop,
    First,
    Last,
    BoundedScan,
}
pub const OPERATIONS: [Operation; 17] = [
    Operation::Control,
    Operation::Get,
    Operation::Miss,
    Operation::Replace,
    Operation::Scan,
    Operation::Insert,
    Operation::Remove,
    Operation::Contains,
    Operation::ReverseScan,
    Operation::KeysScan,
    Operation::Grow,
    Operation::Shrink,
    Operation::RemoveMiss,
    Operation::Reinsert,
    Operation::First,
    Operation::Last,
    Operation::BoundedScan,
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
    pub pattern: Pattern,
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
    assert!((1..=MAX_ROWS).contains(&request.rows));
    assert!((1..=32).contains(&request.count));
    assert!(request.payload <= 65_536);
    let mut state = request.seed;
    let cdf = if request.pattern == Pattern::Zipf {
        let mut sum = 0.0;
        (1..=request.rows)
            .map(|rank| {
                sum += 1.0 / f64::from(rank);
                sum
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    (0..request.count)
        .map(|index| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            match request.operation {
                Operation::Miss | Operation::Insert | Operation::RemoveMiss | Operation::Push => {
                    u64::from(request.rows) + u64::from(index)
                }
                _ => {
                    if request.pattern == Pattern::Zipf {
                        let fraction =
                            f64::from((state >> 32) as u32) / (f64::from(u32::MAX) + 1.0);
                        return cdf.partition_point(|&sum| sum <= fraction * cdf.last().unwrap())
                            as u64;
                    }
                    (state >> 32)
                        % u64::from(if request.pattern == Pattern::Hot16 {
                            request.rows.min(16)
                        } else {
                            request.rows
                        })
                }
            }
        })
        .collect()
}
/// Replacement-size variation is shared by canisters and the independent model.
pub fn payload_for(r: Request) -> u32 {
    match r.operation {
        Operation::Grow => (r.payload.max(32) * 4).min(65_536),
        Operation::Shrink => 0,
        _ => r.payload,
    }
}
pub fn replacement(op: Operation) -> bool {
    matches!(
        op,
        Operation::Replace | Operation::Grow | Operation::Shrink | Operation::Set
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn largest_matrix_responses_fit_protocol_budgets() {
        let rows: Vec<_> = (0..128).map(|id| (id, value(id, 4096, false))).collect();
        let encoded = ic_wire::native::encode_response(3, &rows, LIMITS).unwrap();
        let decoded: Vec<(u64, Vec<u8>)> =
            ic_wire::native::decode_response(3, &encoded, LIMITS).unwrap();
        assert_eq!(decoded, rows);
        let response = Response {
            instructions: 0,
            stable_pages: 0,
            length: 1,
            entries: (0..32)
                .map(|id| (id, Some(value(id, 16_384, false))))
                .collect(),
        };
        let encoded = ic_wire::native::encode_response(RUN_ID, &response, LIMITS).unwrap();
        let decoded: Response = ic_wire::native::decode_response(RUN_ID, &encoded, LIMITS).unwrap();
        assert_eq!(decoded.entries, response.entries);
    }
    #[test]
    fn traces_are_bounded_repeatable_and_missing_keys_are_disjoint() {
        for pattern in [Pattern::Uniform, Pattern::Hot16, Pattern::Zipf] {
            for operation in OPERATIONS {
                let r = Request {
                    case: Case::U64Bytes,
                    operation,
                    rows: 101,
                    payload: 1024,
                    count: 32,
                    seed: 7,
                    pattern,
                };
                let a = keys(r);
                assert_eq!(a, keys(r));
                assert_eq!(a.len(), 32);
                assert!(a.iter().all(|&key| {
                    if matches!(
                        operation,
                        Operation::Miss
                            | Operation::Insert
                            | Operation::RemoveMiss
                            | Operation::Push
                    ) {
                        key >= 101
                    } else {
                        key < if pattern == Pattern::Hot16 { 16 } else { 101 }
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
