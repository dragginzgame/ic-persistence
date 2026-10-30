//! custody contracts.
use super::*;

#[test]
fn fixture_cells_and_detached_handles_refuse_duplicate_setup_without_writes() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.custody.v1",
        config,
        64,
    )
    .unwrap();
    cell.try_set(vec![42]).unwrap();
    let before = backing.borrow().clone();
    let refuse_duplicate = || {
        for result in [
            FixtureCell::create_bounded(
                backing.clone(),
                "fixture",
                "fixture.",
                "fixture.custody.v1",
                config,
                64,
            ),
            FixtureCell::open_bounded(
                backing.clone(),
                "fixture",
                "fixture.",
                "fixture.custody.v1",
                config,
                64,
            ),
        ] {
            assert!(matches!(
                result,
                Err(FixtureCellError::Setup(FixtureRuntimeError::Construction(
                    crate::RuntimeConstructionError::BackingAlreadyClaimed
                )))
            ));
        }
        assert!(
            *backing.borrow() == before,
            "duplicate setup changed backing bytes"
        );
    };
    refuse_duplicate();
    let memory = cell.into_memory();
    refuse_duplicate();
    drop(memory);
    let reopened = FixtureCell::open_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.custody.v1",
        config,
        64,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), &[42]);
}
