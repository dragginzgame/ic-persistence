//! Ledger transitions contracts.

use super::*;

#[test]
fn allocation_record_accessors_expose_read_only_views() {
    let ledger = AllocationLedger::new(1, vec![allocation_record("app.users.v1", 100)]).unwrap();
    assert_eq!(ledger.records().len(), 1);
    let record = &ledger.records()[0];
    assert_eq!(record.stable_key().as_str(), "app.users.v1");
    assert_eq!(record.slot().id(), 100);
}

#[test]
fn stage_generation_records_new_allocations() {
    let declarations = vec![declaration("app.users.v1", 100)];
    let snapshot = snapshot_from(declarations);

    let staged = stage(ledger(), &snapshot).expect("staged generation");

    assert_eq!(staged.current_generation, 4);
    assert_eq!(staged.records().len(), 1);
}

#[test]
fn stage_generation_allows_empty_generation_boundary() {
    let snapshot = snapshot_from(Vec::new());

    let staged = stage(ledger(), &snapshot).expect("empty snapshot generation");

    assert_eq!(staged.current_generation, 4);
    assert_eq!(staged.records(), []);
}

#[test]
fn stage_generation_rejects_same_key_different_slot() {
    let mut ledger = committed_ledger(3);
    ledger.records = vec![allocation_record("app.users.v1", 100)];
    ledger.validate_integrity().unwrap();
    let snapshot = snapshot_from(vec![declaration("app.users.v1", 101)]);

    let err = stage(ledger, &snapshot).expect_err("stable key cannot move slots");

    assert!(matches!(
        err,
        crate::RuntimeBootstrapError::Validation(
            crate::AllocationValidationError::StableKeySlotConflict { .. }
        )
    ));
}

#[test]
fn stage_generation_rejects_same_slot_different_key() {
    let mut ledger = ledger();
    ledger.records = vec![allocation_record("app.users.v1", 100)];
    let snapshot = snapshot_from(vec![declaration("app.orders.v1", 100)]);

    let err = stage(ledger, &snapshot).expect_err("slot cannot be reused by another key");

    assert!(matches!(
        err,
        crate::RuntimeBootstrapError::Validation(
            crate::AllocationValidationError::SlotStableKeyConflict { .. }
        )
    ));
}

#[test]
fn stage_generation_preserves_omitted_records() {
    let first = snapshot_from(vec![
        declaration("app.users.v1", 100),
        declaration("app.orders.v1", 101),
    ]);
    let second = snapshot_from(vec![declaration("app.users.v1", 100)]);

    let staged = stage(ledger(), &first).expect("first generation");
    let staged = stage(staged, &second).expect("second generation");

    assert_eq!(staged.current_generation, 5);
    assert_eq!(staged.records().len(), 2);
    let omitted = staged
        .records()
        .iter()
        .find(|record| record.stable_key.as_str() == "app.orders.v1")
        .expect("omitted record");
    assert_eq!(omitted.slot().id(), 101);
}

#[test]
fn stage_generation_reuses_existing_record() {
    let staged = AllocationLedger::new(
        4,
        vec![crate::testing::allocation_record(&declaration(
            "app.future_store.v1",
            100,
        ))],
    )
    .unwrap();
    let active = snapshot_from(vec![declaration("app.future_store.v1", 100)]);

    let staged = stage(staged, &active).expect("active generation");
    let record = &staged.records()[0];

    assert_eq!(record.slot().id(), 100);
}

#[test]
fn transition_matrix_preserves_committed_invariants() {
    fn visit(ledger: AllocationLedger, depth: usize) {
        ledger.validate_integrity().unwrap();
        if depth == 0 {
            return;
        }
        for declarations in [
            vec![],
            vec![declaration("app.users.v1", 100)],
            vec![declaration("app.users.v1", 100)],
            vec![declaration("app.orders.v1", 101)],
            vec![declaration("app.other.v1", 100)],
        ] {
            let next = stage(ledger.clone(), &snapshot_from(declarations))
                .unwrap_or_else(|_| ledger.clone());
            visit(next, depth - 1);
        }
    }
    visit(AllocationLedger::empty_genesis(), 4);
    let record = crate::testing::allocation_record(&declaration("app.users.v1", 100));
    visit(AllocationLedger::new(1, vec![record]).unwrap(), 4);
}

#[test]
fn validate_integrity_rejects_duplicate_stable_keys() {
    let mut ledger = ledger();
    for second_id in [101, 100] {
        ledger.records = vec![
            allocation_record("app.users.v1", 100),
            allocation_record("app.users.v1", second_id),
        ];

        let err = ledger
            .validate_integrity()
            .expect_err("duplicate key precedes duplicate slot");

        assert_eq!(
            err,
            LedgerIntegrityError::DuplicateStableKey {
                stable_key: StableKey::parse("app.users.v1").unwrap(),
            }
        );
    }
}

#[test]
fn validate_integrity_rejects_duplicate_slots() {
    let mut ledger = ledger();
    ledger.records = vec![
        allocation_record("app.users.v1", 100),
        allocation_record("app.orders.v1", 100),
    ];

    let err = ledger.validate_integrity().expect_err("duplicate slot");

    assert_eq!(
        err,
        LedgerIntegrityError::DuplicateSlot {
            slot: MemoryManagerSlot::new(100).unwrap(),
        }
    );
}
