//! setup contracts.
use super::*;

#[test]
fn cell_keeps_existing_runtime_bytes_and_bucket_configuration() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    // Construct through the ordinary substrate used by independently owned hosts.
    let request = MemoryRequest::new("fixture", "fixture.journal.v1").unwrap();
    let pool = MemoryAllocationPool::new(
        vec![MemoryAuthority::new("fixture", "fixture.").unwrap()],
        vec![],
    )
    .unwrap();
    let declarations = SealedDeclarationSnapshot::new(vec![request]).unwrap();
    let mut runtime = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
    runtime
        .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    let mut original = Cell::init(
        runtime.open_memory("fixture.journal.v1").unwrap(),
        Vec::new(),
    );
    original.set(vec![0, 255, 42]);
    drop(original);
    drop(runtime);

    let maximum = 17 * 65_536;
    let mut cell = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.journal.v1",
        config,
        maximum,
    )
    .unwrap();
    assert_eq!(cell.bytes(), &[0, 255, 42]);
    // Force another bucket after the constructor's runtime has been dropped.
    let bytes = vec![91; maximum];
    cell.try_set(bytes.clone()).unwrap();
    assert_eq!(cell.bytes(), bytes);
    drop(cell);
    let runtime = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
    assert_eq!(runtime.memory_manager_config(), config);
    drop(runtime);
    let reopened = FixtureCell::open_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.journal.v1",
        config,
        maximum,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), bytes);
}

#[test]
fn fresh_empty_cell_is_distinct_from_incompatible_retained_configuration() {
    let backing = VectorMemory::default();
    let mut cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.journal.v1",
        MemoryManagerConfig::new(16).unwrap(),
        64,
    )
    .unwrap();
    assert_eq!(cell.bytes(), &[0_u8; 0]);
    cell.try_set(vec![37]).unwrap();
    drop(cell);
    let before = backing.borrow().clone();
    let failure = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.journal.v1",
        MemoryManagerConfig::new(128).unwrap(),
        64,
    );
    assert!(matches!(
        failure,
        Err(FixtureCellError::Setup(FixtureRuntimeError::Construction(
            _
        )))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn fallible_setup_keeps_payload_and_transfers_the_admitted_memory() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
        64,
    )
    .unwrap();
    cell.try_set(b"committed journal".to_vec()).unwrap();
    let memory = cell.into_memory();
    assert_eq!(Cell::init(memory, Vec::new()).get(), b"committed journal");
    let reopened = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
        64,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), b"committed journal");
    drop(reopened);
    let before = backing.borrow().clone();
    let refused = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        MemoryManagerConfig::new(32).unwrap(),
        64,
    );
    assert!(matches!(
        refused,
        Err(FixtureCellError::Setup(FixtureRuntimeError::Construction(
            _
        )))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn fallible_setup_preserves_typed_request_grant_and_bootstrap_errors() {
    let config = MemoryManagerConfig::new(16).unwrap();
    let backing = VectorMemory::default();
    let refused = FixtureCell::create_bounded(
        backing.clone(),
        "",
        "fixture.",
        "fixture.jobs.v1",
        config,
        64,
    );
    assert!(matches!(
        refused,
        Err(FixtureCellError::Setup(FixtureRuntimeError::Declaration(_)))
    ));
    assert_eq!(backing.size(), 0);
    let refused = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "invalid",
        "fixture.jobs.v1",
        config,
        64,
    );
    assert!(matches!(
        refused,
        Err(FixtureCellError::Setup(
            FixtureRuntimeError::AllocationPool(_)
        ))
    ));
    assert_eq!(backing.size(), 0);
    let refused =
        FixtureCell::create_bounded(backing, "fixture", "other.", "fixture.jobs.v1", config, 64);
    assert!(matches!(
        refused,
        Err(FixtureCellError::Setup(FixtureRuntimeError::Bootstrap(_)))
    ));
}

#[test]
fn fallible_setup_refuses_non_cell_history_without_replacing_its_header() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
        64,
    )
    .unwrap();
    let memory = cell.into_memory();
    memory.write(0, b"NOT\x01");
    let mut before = [0; crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut before);
    drop(memory);
    let refused = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
        64,
    );
    assert!(matches!(
        refused,
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::NotStableCell
        ))
    ));
    let memory = reopened_memory(backing, "fixture.jobs.v1", config);
    let mut after = [0; crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut after);
    assert_eq!(after, before);
}

#[test]
fn fixture_runtime_keeps_named_allocations_and_returns_the_authority() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let keys = ["fixture.map.v1", "fixture.cell.v1"];
    let runtime = fixture_runtime(backing.clone(), "fixture", "fixture.", &keys, config).unwrap();
    assert!(matches!(
        runtime.open_memory("fixture.unknown.v1"),
        Err(crate::RuntimeOpenError::StableKeyNotCommitted(_))
    ));
    let map = runtime.open_memory(keys[0]).unwrap();
    let cell = runtime.open_memory(keys[1]).unwrap();
    assert_eq!(map.size(), 0, "setup must not initialize a collection");
    assert_eq!(cell.size(), 0);
    assert_eq!(map.grow(1).unwrap(), 0);
    map.write(0, b"map bytes");
    assert_eq!(cell.size(), 0, "allocations must remain distinct");
    drop((runtime, map, cell));
    let runtime = fixture_runtime(backing.clone(), "fixture", "fixture.", &keys, config).unwrap();
    let map = runtime.open_memory(keys[0]).unwrap();
    let mut bytes = [0; 9];
    map.read(0, &mut bytes);
    assert_eq!(&bytes, b"map bytes");
    drop((runtime, map));
    let before = backing.borrow().clone();
    assert!(matches!(
        fixture_runtime(
            backing.clone(),
            "fixture",
            "fixture.",
            &keys,
            MemoryManagerConfig::new(32).unwrap()
        ),
        Err(FixtureRuntimeError::Construction(_))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn fixture_runtime_preserves_setup_error_categories() {
    let config = MemoryManagerConfig::new(16).unwrap();
    let backing = VectorMemory::default();
    assert!(matches!(
        fixture_runtime(backing.clone(), "", "fixture.", &["fixture.map.v1"], config),
        Err(FixtureRuntimeError::Declaration(_))
    ));
    assert!(matches!(
        fixture_runtime(
            backing.clone(),
            "fixture",
            "fixture.",
            &["fixture.map.v1", "fixture.map.v1"],
            config
        ),
        Err(FixtureRuntimeError::Declaration(_))
    ));
    assert!(matches!(
        fixture_runtime(
            backing.clone(),
            "fixture",
            "invalid",
            &["fixture.map.v1"],
            config
        ),
        Err(FixtureRuntimeError::AllocationPool(_))
    ));
    assert_eq!(
        backing.size(),
        0,
        "invalid declarations/grants must not initialize backing"
    );
    assert!(matches!(
        fixture_runtime(backing, "fixture", "other.", &["fixture.map.v1"], config),
        Err(FixtureRuntimeError::Bootstrap(_))
    ));
}
