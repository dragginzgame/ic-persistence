//! bounded contracts.
use super::*;

#[test]
fn bounded_cell_preserves_history_and_refuses_overwrites_or_oversized_writes() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    )
    .unwrap();
    cell.try_set(vec![42; 16_384]).unwrap();
    let before = backing.borrow().clone();
    assert!(matches!(
        cell.try_set(vec![0; 16_385]),
        Err(FixtureCellError::ValueTooLarge {
            size: 16_385,
            maximum: 16_384,
        })
    ));
    assert_eq!(cell.bytes(), &[42; 16_384]);
    assert_eq!(*backing.borrow(), before);
    let memory = cell.into_memory();
    let mut payload_before = vec![0; 16_384 + crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut payload_before);
    drop(memory);
    assert!(matches!(
        FixtureCell::create_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            16_384,
        ),
        Err(FixtureCellError::Occupied)
    ));
    assert!(matches!(
        FixtureCell::open_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            16_383,
        ),
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::TooLarge { value_len: 16_384 }
        ))
    ));
    let memory = reopened_memory(backing.clone(), "fixture.consumer.v1", config);
    let mut payload_after = vec![0; payload_before.len()];
    memory.read(0, &mut payload_after);
    assert_eq!(payload_after, payload_before);
    drop(memory);
    let mut reopened = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), &[42; 16_384]);
    // A short replacement exposes only the new logical value.
    reopened.try_set(vec![7]).unwrap();
    drop(reopened);
    let reopened = FixtureCell::open_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        1,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), &[7]);
}

#[test]
fn bounded_open_refuses_missing_and_malformed_history_without_replacement() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    assert!(matches!(
        FixtureCell::open_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            16_384,
        ),
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::NotStableCell
        ))
    ));
    assert_eq!(backing.size(), 0);
    let cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    )
    .unwrap();
    let mut memory = cell.into_memory();
    // Assert each malformed header survives exactly, including impossible lengths.
    for header in [
        *b"BAD\x01\x00\x00\x00\x00",
        *b"SCL\x02\x00\x00\x00\x00",
        *b"SCL\x01\xff\xff\xff\xff",
    ] {
        memory.write(0, &header);
        drop(memory);
        assert!(matches!(
            FixtureCell::open_bounded(
                backing.clone(),
                "fixture",
                "fixture.",
                "fixture.consumer.v1",
                config,
                16_384,
            ),
            Err(FixtureCellError::Payload(_))
        ));
        memory = reopened_memory(backing.clone(), "fixture.consumer.v1", config);
        let mut after = [0; crate::STABLE_CELL_HEADER_SIZE];
        memory.read(0, &mut after);
        assert_eq!(after, header);
    }
}

#[test]
fn bounded_open_does_not_initialize_a_missing_cell_in_an_existing_manager() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut runtime = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
    let declarations = SealedDeclarationSnapshot::new(vec![
        MemoryRequest::new("fixture", "fixture.consumer.v1").unwrap(),
    ])
    .unwrap();
    let pool = MemoryAllocationPool::new(
        vec![MemoryAuthority::new("fixture", "fixture.").unwrap()],
        vec![],
    )
    .unwrap();
    runtime
        .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    assert_eq!(
        runtime.open_memory("fixture.consumer.v1").unwrap().size(),
        0
    );
    drop(runtime);
    assert!(matches!(
        FixtureCell::open_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            0,
        ),
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::NotStableCell
        ))
    ));
    // Fresh admission is still available: failed retained admission created no Cell.
    let mut fresh = FixtureCell::create_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        0,
    )
    .unwrap();
    assert_eq!(fresh.bytes(), [0_u8; 0]);
    fresh.try_set(Vec::new()).unwrap();
    assert!(matches!(
        fresh.try_set(vec![1]),
        Err(FixtureCellError::ValueTooLarge {
            size: 1,
            maximum: 0
        })
    ));
}
