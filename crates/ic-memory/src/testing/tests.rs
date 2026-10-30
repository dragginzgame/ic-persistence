use super::*;
use crate::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryRequest, MemoryRuntime,
    SealedDeclarationSnapshot, VectorMemory,
};

// Inspect retained cell bytes through a fresh governed lease after the previous
// fixture/handle has released its backing claim. No cell decoder or reset runs.
fn reopened_memory(
    backing: VectorMemory,
    stable_key: &str,
    config: MemoryManagerConfig,
) -> RuntimeMemory<VectorMemory> {
    fixture_runtime(backing, "fixture", "fixture.", &[stable_key], config)
        .unwrap()
        .open_memory(stable_key)
        .unwrap()
}

mod bounded;
mod custody;
mod setup;
