// Fixture-only Wire contract compiled by the canister and the native runner.
use ic_wire::Limits;
use serde::{Deserialize, Serialize};

pub const REQUEST: Limits = Limits {
    bytes: 128,
    allocation: 4096,
    nodes: 128,
    depth: 8,
};
pub const RESPONSE: Limits = REQUEST;

pub const INIT_ID: u64 = 0x5354_4f51_0000_0001;
pub const UPGRADE_ID: u64 = 0x5354_4f51_0000_0002;
pub const SNAPSHOT_ID: u64 = 0x5354_4f51_0000_0003;
pub const MUTATE_ID: u64 = 0x5354_4f51_0000_0004;
pub const CELL_FAILURE_ID: u64 = 0x5354_4f51_0000_0005;

///
/// Snapshot
///
/// Current generation and the three retained collection values.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Snapshot {
    pub generation: u64,
    pub values: [u64; 3],
}
