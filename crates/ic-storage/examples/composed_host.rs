//! Host composition over names and namespace grants, with no numeric ID pools.
use ic_storage::{
    AllocationGrants, GenericAllocationPolicy, MemoryAuthority, MemoryRequest, MemoryRuntime,
    SealedDeclarationSnapshot, StableKey, VectorMemory, adapter::StableCell,
};
fn main() {
    let mut host = MemoryRuntime::new(VectorMemory::default()).unwrap();
    let declarations = SealedDeclarationSnapshot::new(vec![
        MemoryRequest::new("host", "host.control.v1").unwrap(),
        MemoryRequest::new("db", "db.rows.v1").unwrap(),
    ])
    .unwrap();
    let grants = AllocationGrants::new(vec![
        MemoryAuthority::new("host", "host.").unwrap(),
        MemoryAuthority::new("db", "db.").unwrap(),
    ])
    .unwrap();
    host.bootstrap(&declarations, &grants, &GenericAllocationPolicy)
        .unwrap();
    let value = StableCell::create(
        host.open_memory(&StableKey::parse("host.control.v1").unwrap())
            .unwrap(),
        7_u64,
    )
    .unwrap();
    assert_eq!(*value.get(), 7);
}
