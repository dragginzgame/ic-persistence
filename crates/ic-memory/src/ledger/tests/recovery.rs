//! Ledger recovery contracts.

use super::*;

#[test]
fn repeated_commits_keep_metadata_bounded() {
    let mut store = LedgerCommitStore::default();
    store
        .commit(&AllocationLedger::new(65_530, Vec::new()).unwrap())
        .unwrap();
    let mut largest_payload = 0;
    for version in 1_u32..=1024 {
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.users.v1", 100)]);
        let recovered = store.recover().unwrap();
        let (next, _) =
            stage_generation(recovered, snapshot, &crate::GenericAllocationPolicy).unwrap();
        store.commit_generation(&next).unwrap();
        assert_eq!(next.current_generation(), 65_530 + u64::from(version));
        let recovered = store.recover().unwrap();
        assert_eq!(recovered.ledger().records().len(), 1);
        largest_payload = largest_payload.max(store.authoritative().unwrap().payload().len());
    }
    // Wire integer widths may grow; the number of records and metadata do not.
    assert!(largest_payload < 200);
    assert!(
        crate::stable_cell::EncodedLedgerRecord::new(&store)
            .unwrap()
            .len()
            < 600
    );
}

#[test]
fn largest_current_record_set_fits_writer_and_reader_bounds() {
    let records = (0..255_u8)
        .map(|id| {
            let key = format!("app.store{id:03}.{}.v1", "x".repeat(112));
            assert_eq!(key.len(), 128);
            let declaration = declaration(&key, id);
            AllocationRecord::from_declaration(&declaration)
        })
        .collect();
    let ledger = AllocationLedger::new(u64::MAX - 1, records).unwrap();
    let payload = encoded_payload(&ledger);
    assert!(payload.len() <= crate::constants::MAX_COMMITTED_PAYLOAD_BYTES);
    let mut store = LedgerCommitStore::default();
    store.commit(&ledger).unwrap();
    let next = AllocationLedger::new(u64::MAX, ledger.records().to_vec()).unwrap();
    store.commit(&next).unwrap();
    let record = store;
    let bytes = crate::stable_cell::EncodedLedgerRecord::new(&record)
        .unwrap()
        .into_bytes();
    assert!(bytes.len() <= crate::constants::MAX_LEDGER_RECORD_BYTES);
    let decoded = crate::decode_stable_cell_commit_store(&bytes).unwrap();
    assert_eq!(decoded.recover().unwrap().ledger(), &next);
}

#[test]
fn validated_counter_overflow_rejects_without_commit_mutation() {
    let mut store = LedgerCommitStore::default();
    store
        .commit(&AllocationLedger::new(u64::MAX, Vec::new()).unwrap())
        .unwrap();
    let before = store.clone();
    let recovered = store.recover().unwrap();
    let result = stage_generation(
        recovered,
        DeclarationSnapshot::for_test(Vec::new()),
        &crate::GenericAllocationPolicy,
    );
    assert!(matches!(
        result,
        Err(crate::RuntimeBootstrapError::Staging(
            AllocationStageError::GenerationOverflow {
                generation: u64::MAX
            }
        ))
    ));
    assert_eq!(store, before);
}

#[test]
fn recovery_rejects_invalid_stable_key_during_ledger_decode() {
    let mut ledger = committed_ledger(1);
    ledger.records.push(allocation_record("app.users.v1", 100));
    let mut bytes = codec::encode_ledger(&ledger).expect("encode ledger");
    let key_start = bytes
        .windows(b"app.users.v1".len())
        .position(|window| window == b"app.users.v1")
        .expect("encoded stable key");
    bytes[key_start] = b'A';
    assert!(decode_ledger(&bytes, 1).is_err());
    let payload = bytes;
    let mut physical = LedgerCommitStore::default();
    physical.commit_payload_at_generation(1, payload).unwrap();
    let store = physical;
    let before = store.clone();
    assert!(matches!(store.recover(), Err(LedgerCommitError::Codec(_))));
    assert_eq!(store, before, "malformed-key recovery must preserve slots");
}

#[test]
fn ledger_decode_rejects_unusable_memory_manager_slot() {
    let bytes = crate::test_wire::to_vec(&vec![("app.users.v1", 255_u8)]).unwrap();
    assert!(decode_ledger(&bytes, 1).is_err());
}

#[test]
fn integrity_rejects_nonempty_genesis_before_commit() {
    let invalid = AllocationLedger {
        current_generation: 0,
        records: vec![allocation_record("app.users.v1", 100)],
    };
    assert_eq!(
        invalid.validate_integrity(),
        Err(LedgerIntegrityError::NonemptyGenesis)
    );
    let mut store = LedgerCommitStore::default();
    let before = store.clone();
    assert_eq!(
        store.commit(&invalid),
        Err(LedgerCommitError::Integrity(
            LedgerIntegrityError::NonemptyGenesis
        ))
    );
    assert_eq!(store, before);
}

#[test]
fn ledger_commit_store_rejects_invalid_ledger_before_write() {
    let mut store = LedgerCommitStore::default();
    let mut invalid = ledger();
    invalid.records = vec![
        allocation_record("app.users.v1", 100),
        allocation_record("app.orders.v1", 100),
    ];

    let err = store.commit(&invalid).expect_err("invalid ledger");

    assert!(matches!(
        err,
        LedgerCommitError::Integrity(LedgerIntegrityError::DuplicateSlot { .. })
    ));
    assert!(store.current().is_none());
}

#[test]
fn ledger_commit_store_recovers_latest_committed_ledger() {
    let mut store = LedgerCommitStore::default();
    let first = committed_ledger(1);
    let second = committed_ledger(2);

    store.commit(&first).expect("first commit");
    store.commit(&second).expect("second commit");
    let recovered = store.recover().expect("recovered ledger");

    assert_eq!(recovered.current_generation(), 2);
}

#[test]
fn ledger_commit_store_encodes_direct_logical_wire() {
    let mut store = LedgerCommitStore::default();
    let ledger = committed_ledger(1);

    store.commit(&ledger).expect("commit");

    let committed = store.authoritative().expect("authoritative");
    assert_eq!(
        committed.payload(),
        crate::test_wire::to_vec(&ledger.records()).unwrap(),
        "commit payload matches canonical record-array Wire"
    );
}

#[test]
fn ledger_commit_store_rejects_malformed_logical_wire() {
    let committed = CommittedGenerationBytes::new(1, vec![b'X'; 28]);
    let store = LedgerCommitStore {
        current: Some(committed),
    };

    let err = store.recover().expect_err("bad logical Wire");

    assert!(matches!(err, LedgerCommitError::Codec(_)));
}

#[test]
fn recovery_and_initialization_preserve_physical_diagnostics_and_logical_failures() {
    let genesis = encoded_payload(&committed_ledger(0));
    let committed = CommittedGenerationBytes::new(0, genesis.clone());
    let mut corrupt = committed.clone();
    corrupt.checksum ^= 1;
    let mut unsupported = committed.clone();
    unsupported.commit_marker = u64::MAX;
    let cases = [
        ("empty", None),
        ("current", Some(committed)),
        ("corrupt", Some(corrupt)),
        ("unsupported format", Some(unsupported)),
        (
            "invalid Wire",
            Some(CommittedGenerationBytes::new(0, vec![255])),
        ),
        (
            "checked nonzero generation with empty records",
            Some(CommittedGenerationBytes::new(1, genesis)),
        ),
        (
            "invalid committed integrity",
            Some(CommittedGenerationBytes::new(
                0,
                encoded_payload(&committed_fixture_ledger()),
            )),
        ),
    ];

    for (label, current) in cases {
        let empty = current.is_none();
        let store = LedgerCommitStore { current };
        let before = store.clone();
        let (recovered, diagnostic) = store.recover_with_diagnostic();
        assert_eq!(recovered, store.recover(), "{label}");
        assert_eq!(diagnostic, store.diagnostic(), "{label}");
        assert_eq!(store, before, "{label}");

        let initializing = store.clone();
        let initialized = initializing.recover_or_genesis();
        if empty {
            let initialized = initialized.expect("empty store accepts genesis");
            assert_eq!(initialized.current_generation(), 0);
            assert_eq!(initialized.ledger().records(), []);
        } else {
            assert_eq!(initialized, recovered, "{label}");
        }
        assert_eq!(initializing, before, "{label}");

        if let Err(LedgerCommitError::Recovery(error)) = recovered {
            let expected = match error {
                CommitRecoveryError::NoValidGeneration => crate::CommitSnapshotDiagnostic::Empty,
                CommitRecoveryError::InvalidSnapshot { generation } => {
                    crate::CommitSnapshotDiagnostic::Invalid { generation }
                }
                CommitRecoveryError::UnsupportedFormat { .. } => {
                    crate::CommitSnapshotDiagnostic::Invalid {
                        generation: before.current().unwrap().generation(),
                    }
                }
                _ => panic!("unexpected physical recovery failure: {error}"),
            };
            assert_eq!(diagnostic, expected, "{label}");
        } else {
            assert!(
                matches!(diagnostic, crate::CommitSnapshotDiagnostic::Valid { .. }),
                "{label}"
            );
        }
    }
}

#[test]
fn ledger_commit_store_recovers_current_format_genesis_and_first_real_commit() {
    let mut store = LedgerCommitStore::default();
    let recovered = store
        .recover_or_genesis()
        .expect("current-format genesis ledger");
    assert_eq!(recovered.current_generation(), 0);

    assert!(store.current().is_none());
    assert!(matches!(
        store.recover(),
        Err(LedgerCommitError::Recovery(
            CommitRecoveryError::NoValidGeneration
        ))
    ));

    let first = stage(
        recovered.ledger().clone(),
        &snapshot_from(vec![declaration("app.users.v1", 100)]),
    )
    .expect("first real generation");
    let recovered = store.commit(&first).expect("first commit");

    assert_eq!(recovered, store.recover().unwrap());
    assert_eq!(recovered.current_generation(), 1);
}

#[test]
fn ledger_commit_store_rejects_corrupt_current_snapshot() {
    let mut store = LedgerCommitStore::default();
    let genesis = committed_ledger(0);
    store.commit(&genesis).expect("genesis commit");
    let first = stage(
        genesis,
        &snapshot_from(vec![declaration("app.users.v1", 100)]),
    )
    .expect("first generation");
    let first = store.commit(&first).expect("first commit");
    let second = stage(
        first.ledger().clone(),
        &snapshot_from(vec![declaration("app.users.v1", 100)]),
    )
    .expect("second generation");

    store.commit(&second).expect("second commit");
    store.current.as_mut().expect("latest slot").checksum ^= 1;

    let err = store
        .recover()
        .expect_err("corrupt latest must not roll back");

    assert_eq!(
        err,
        LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 2 })
    );
}

#[test]
fn ledger_commit_store_rejects_corrupt_current_ledger() {
    let mut store = LedgerCommitStore::default();
    let first = committed_ledger(1);
    let second = committed_ledger(2);

    store.commit(&first).expect("first commit");
    store.write_corrupt_ledger(&second).expect("corrupt write");
    let err = store.recover().expect_err("corrupt slot");

    assert_eq!(
        err,
        LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 2 })
    );
}

#[test]
fn ledger_commit_store_recovers_generation_from_protected_header() {
    let payload = encoded_payload(&committed_fixture_ledger());
    for generation in [1, 7, u64::MAX] {
        let store = LedgerCommitStore {
            current: Some(CommittedGenerationBytes::new(generation, payload.clone())),
        };
        let recovered = store.recover().unwrap();
        assert_eq!(recovered.current_generation(), generation);
        assert_eq!(
            recovered.ledger().records(),
            committed_fixture_ledger().records()
        );
    }
}

#[test]
fn ledger_commit_store_rejects_non_next_logical_generation() {
    let mut store = LedgerCommitStore::default();
    store.commit(&committed_ledger(1)).expect("first commit");

    let err = store
        .commit(&committed_ledger(3))
        .expect_err("skipped generation");

    assert_eq!(
        err,
        LedgerCommitError::Recovery(CommitRecoveryError::UnexpectedGeneration {
            expected: 2,
            actual: 3
        })
    );
}

#[test]
fn ledger_commit_store_rejects_corrupt_store_even_with_genesis() {
    let mut store = LedgerCommitStore::default();
    store
        .write_corrupt_ledger(&ledger())
        .expect("corrupt write");

    let err = store.recover_or_genesis().expect_err("corrupt state");

    assert!(matches!(
        err,
        LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 3 })
    ));
}

#[test]
fn record_count_boundary_and_malicious_count_reject_on_recovery() {
    let mut ledger = committed_ledger(1);
    ledger.records = (0..255_u8)
        .map(|id| allocation_record(&format!("app.slot{id}.v1"), id))
        .collect();
    let mut store = LedgerCommitStore::default();
    store.commit(&ledger).unwrap();
    assert_eq!(store.recover().unwrap().ledger().records().len(), 255);
    ledger.records.push(allocation_record("app.extra.v1", 254));
    assert!(matches!(
        store.commit(&ledger),
        Err(LedgerCommitError::Integrity(
            LedgerIntegrityError::LimitExceeded { .. }
        ))
    ));
    let payload = crate::test_wire::to_vec(ledger.records()).unwrap();
    let invalid = LedgerCommitStore {
        current: Some(CommittedGenerationBytes::new(1, payload)),
    };
    assert!(invalid.recover().is_err());
    assert!(invalid.recover_or_genesis().is_err());
    for bytes in [vec![255; 10], vec![128, 2], vec![1]] {
        let payload = bytes;
        let invalid = LedgerCommitStore {
            current: Some(CommittedGenerationBytes::new(1, payload)),
        };
        assert!(matches!(
            invalid.recover(),
            Err(LedgerCommitError::Codec(_))
        ));
    }
}
