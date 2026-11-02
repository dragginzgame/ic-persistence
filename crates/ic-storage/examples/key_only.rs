//! Run with `cargo run --example key_only`.
use ic_storage::VectorMemory;
use ic_storage::{
    AllocationGrants, GenericAllocationPolicy, MemoryAuthority, MemoryRequest, MemoryRuntime,
    SealedDeclarationSnapshot,
};
fn request(owner: &str, key: &str) -> MemoryRequest {
    MemoryRequest::new(owner, key).unwrap()
}
fn main() {
    // The final host owns one pool. Owners receive namespaces, not numeric ranges.
    let pool = AllocationGrants::new(vec![
        MemoryAuthority::new("app", "app.").unwrap(),
        MemoryAuthority::new("db", "db.").unwrap(),
    ])
    .unwrap();
    let declarations = SealedDeclarationSnapshot::new(vec![
        request("app", "app.control.v1"),
        request("db", "db.journal.v1"),
        request("db", "db.rows.v1"),
    ])
    .unwrap();
    let backing = VectorMemory::default();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    host.bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    let id = host
        .allocation_id(&ic_storage::StableKey::parse("db.rows.v1").unwrap())
        .unwrap();
    let rows = host
        .open_memory(&ic_storage::StableKey::parse("db.rows.v1").unwrap())
        .unwrap();
    rows.grow(1).unwrap();
    rows.write(0, b"rows");
    drop(rows);
    drop(host);
    let mut host = MemoryRuntime::new(backing).unwrap();
    host.bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    // Library adoption verifies only its requirements and does not bootstrap again.
    let requirements = SealedDeclarationSnapshot::new(vec![
        request("db", "db.journal.v1"),
        request("db", "db.rows.v1"),
    ])
    .unwrap();
    host.verify_authority(&requirements, "db").unwrap();
    assert_eq!(
        host.allocation_id(&ic_storage::StableKey::parse("db.rows.v1").unwrap()),
        Some(id)
    );
    let mut bytes = [0; 4];
    host.open_memory(&ic_storage::StableKey::parse("db.rows.v1").unwrap())
        .unwrap()
        .read(0, &mut bytes);
    assert_eq!(&bytes, b"rows");
}
