//! Shared declarations and pools for runtime unit tests.

use crate::{MemoryAllocationPool, MemoryAuthority, MemoryRequest, SealedDeclarationSnapshot};

pub(super) fn snapshot(keys: &[&str], owner: &str) -> SealedDeclarationSnapshot {
    SealedDeclarationSnapshot::new(
        keys.iter()
            .map(|key| MemoryRequest::new(owner, key).unwrap())
            .collect(),
    )
    .unwrap()
}

pub(super) fn app_pool() -> MemoryAllocationPool {
    MemoryAllocationPool::new(vec![MemoryAuthority::new("app", "app.").unwrap()], vec![]).unwrap()
}

pub(super) fn declarations() -> SealedDeclarationSnapshot {
    snapshot(&["runtime_tests.rows.v1"], "runtime_tests")
}

pub(super) fn pool() -> MemoryAllocationPool {
    MemoryAllocationPool::new(
        vec![MemoryAuthority::new("runtime_tests", "runtime_tests.").unwrap()],
        (10..=119).chain(121..=254).collect(),
    )
    .unwrap()
}
