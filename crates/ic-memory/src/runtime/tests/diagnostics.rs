//! Diagnostics contracts.

use super::*;

#[test]
fn doctor_keeps_memory_observations_before_policy_callbacks() {
    struct GrowingPolicy<'a>(&'a MemoryRuntime<VectorMemory>);

    impl RuntimeBootstrapPolicy for GrowingPolicy<'_> {
        type Error = Infallible;

        fn validate_allocation(
            &self,
            _key: &StableKey,
            _slot: &MemoryManagerSlot,
        ) -> Result<(), Self::Error> {
            self.0
                .open_memory("runtime_tests.rows.v1")
                .unwrap()
                .grow(1)
                .unwrap();
            Ok(())
        }

        fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
            self.0
                .open_memory("runtime_tests.rows.v1")
                .unwrap()
                .grow(1)
                .unwrap();
            PolicyIdentity::new(*b"runtime-tests.growing-doctor....")
        }
    }

    let mut runtime = MemoryRuntime::new_with_config(
        VectorMemory::default(),
        crate::runtime::MemoryManagerConfig::new(1).unwrap(),
    )
    .unwrap();
    runtime
        .bootstrap(&declarations(), &pool(), &GenericAllocationPolicy)
        .unwrap();
    let memory = runtime.open_memory("runtime_tests.rows.v1").unwrap();
    memory.grow(1).unwrap();
    drop(memory);
    let report =
        crate::testing::doctor_report(&runtime, &declarations(), &pool(), &GrowingPolicy(&runtime));
    assert_eq!(
        runtime.open_memory("runtime_tests.rows.v1").unwrap().size(),
        3
    );
    assert_eq!(report.validation, DiagnosticCheck::Passed);
    let record = report
        .ledger
        .unwrap()
        .records
        .into_iter()
        .find(|record| record.allocation.stable_key().as_str() == "runtime_tests.rows.v1")
        .unwrap();
    assert_eq!(record.memory_size.unwrap().wasm_pages, 1);
}

#[test]
fn doctor_and_diagnostics_report_the_same_runtime_lifecycle() {
    let declarations = declarations();
    let mut runtime = empty_runtime();

    assert!(
        !crate::testing::doctor_report(&runtime, &declarations, &pool(), &GenericAllocationPolicy)
            .bootstrapped
    );
    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::NotBootstrapped)
    ));

    runtime
        .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
        .expect("bootstrap");
    let memory = runtime.open_memory("runtime_tests.rows.v1").expect("open");
    memory.grow(2).unwrap();
    let doctor =
        crate::testing::doctor_report(&runtime, &declarations, &pool(), &GenericAllocationPolicy);
    let export = runtime.diagnostic_export().expect("diagnostic export");
    assert!(doctor.bootstrapped);
    assert_eq!(doctor.ledger.as_ref().expect("doctor ledger"), &export);
    assert_eq!(
        doctor.commit_recovery,
        Some(crate::CommitSnapshotDiagnostic::Valid { generation: 1 })
    );
    assert_eq!(
        export
            .records
            .iter()
            .find(|record| { record.allocation.stable_key().as_str() == "runtime_tests.rows.v1" })
            .expect("runtime record")
            .memory_size,
        Some(DiagnosticMemorySize::from_wasm_pages(2))
    );
    assert!(matches!(&doctor.bootstrap_binding, DiagnosticCheck::Passed));
    assert_eq!(
        doctor.tested_policy_identity,
        PolicyIdentity::new(*b"ic-memory.noop-policy...........")
    );
    let established = doctor
        .established_bootstrap_binding
        .as_ref()
        .expect("established runtime binding");
    assert_eq!(
        established.policy_identity,
        PolicyIdentity::new(*b"ic-memory.noop-policy...........")
    );
    let doctor_json = serde_json::to_value(&doctor).expect("doctor diagnostic JSON");
    assert_eq!(doctor_json["bootstrapped"], true);
    assert_eq!(
        doctor_json["tested_policy_identity"],
        serde_json::json!(doctor.tested_policy_identity.as_bytes())
    );
    assert_eq!(
        doctor_json["established_bootstrap_binding"]["policy_identity"],
        doctor_json["tested_policy_identity"]
    );
    assert_eq!(doctor_json["bootstrap_binding"], "Passed");
    assert_eq!(
        doctor_json["commit_recovery"],
        serde_json::json!({"Valid": {"generation": 1}})
    );
    assert_eq!(
        doctor_json["ledger"],
        serde_json::to_value(&export).unwrap()
    );

    let mismatched = crate::testing::doctor_report(
        &runtime,
        &declarations,
        &pool(),
        &IdentityPolicy::new([2; 32]),
    );
    assert!(matches!(
        mismatched.bootstrap_binding,
        DiagnosticCheck::Failed {
            code: DiagnosticCode::RuntimeBinding,
            ..
        }
    ));
    assert!(matches!(mismatched.validation, DiagnosticCheck::Passed));
    assert_eq!(
        mismatched.tested_policy_identity,
        PolicyIdentity::new([2; 32])
    );
    assert_eq!(mismatched.allocation_pool, pool());
    assert_eq!(
        mismatched.established_bootstrap_binding,
        doctor.established_bootstrap_binding
    );
}

#[test]
fn diagnostics_reject_invalid_persisted_slots_before_measuring_sizes() {
    let declarations = SealedDeclarationSnapshot::new(vec![]).unwrap();
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
        .unwrap();
    let record = runtime.ledger_record_from_memory().unwrap();
    let ledger = record.recover().unwrap().into_ledger();
    let payload = crate::test_wire::to_vec(&vec![("ic_memory.ledger.v1", 255_u8)]).unwrap();
    // Keep physical framing/checksums valid so recovery reaches slot validation.
    let mut physical = crate::LedgerCommitStore::default();
    physical
        .commit_payload_at_generation(ledger.current_generation(), payload)
        .unwrap();
    let store = physical;
    let _cell = Cell::new(
        runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID).unwrap(),
        crate::stable_cell::EncodedLedgerRecord::new(&store).unwrap(),
    );
    drop(_cell);
    let before = backing.borrow().clone();

    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::LedgerCommit(
            LedgerCommitError::Codec(_)
        ))
    ));
    let doctor =
        crate::testing::doctor_report(&runtime, &declarations, &pool(), &GenericAllocationPolicy);
    assert!(doctor.ledger.is_none());
    assert!(matches!(
        doctor.validation,
        DiagnosticCheck::NotRun {
            code: DiagnosticCode::LedgerRecovery,
            ..
        }
    ));
    drop(runtime);
    let mut reopened = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        reopened.bootstrap(&declarations, &pool(), &GenericAllocationPolicy),
        Err(crate::runtime::RuntimeBootstrapError::LedgerCommit(
            LedgerCommitError::Codec(_)
        ))
    ));
    assert!(!reopened.is_bootstrapped());
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn doctor_preserves_distinct_record_decode_causes_without_writes() {
    let declarations = SealedDeclarationSnapshot::new(vec![]).unwrap();
    let mut messages = Vec::new();
    let mut trailing = crate::encoding::RECORD_MAGIC.to_vec();
    trailing.extend_from_slice(&[0, 0]);
    let mut malformed = crate::encoding::RECORD_MAGIC.to_vec();
    malformed.push(2);
    for bytes in [trailing, malformed] {
        let cause = crate::decode_stable_cell_commit_store(&bytes)
            .unwrap_err()
            .to_string();
        let backing = VectorMemory::default();
        let runtime = MemoryRuntime::new(backing.clone()).unwrap();
        let memory = runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID).unwrap();
        memory.grow(1).unwrap();
        memory.write(0, crate::STABLE_CELL_MAGIC);
        memory.write(3, &[crate::STABLE_CELL_LAYOUT_VERSION]);
        memory.write(4, &u32::try_from(bytes.len()).unwrap().to_le_bytes());
        memory.write(crate::STABLE_CELL_VALUE_OFFSET, &bytes);
        let before = backing.borrow().clone();

        let report = crate::testing::doctor_report(
            &runtime,
            &declarations,
            &pool(),
            &GenericAllocationPolicy,
        );
        assert!(report.commit_recovery.is_none());
        assert!(report.ledger.is_none());
        let crate::testing::DiagnosticStableCellStatus::Corrupt { failure } =
            report.stable_cell.status
        else {
            panic!("malformed record must be reported as corrupt");
        };
        assert_eq!(failure.code, DiagnosticCode::StableCell);
        assert!(failure.message.contains(&cause));
        messages.push(failure.message);
        assert_eq!(*backing.borrow(), before);
    }
    assert_ne!(messages[0], messages[1]);
}

#[test]
fn doctor_uses_genesis_only_for_empty_commit_storage() {
    let declarations = SealedDeclarationSnapshot::new(vec![]).unwrap();
    let backing = VectorMemory::default();
    let runtime = MemoryRuntime::new(backing.clone()).unwrap();
    for initialized in [false, true] {
        if initialized {
            let _cell = Cell::new(
                runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID).unwrap(),
                crate::stable_cell::EncodedLedgerRecord::new(&crate::LedgerCommitStore::default())
                    .unwrap(),
            );
        }
        let before = backing.borrow().clone();
        let report = crate::testing::doctor_report(
            &runtime,
            &declarations,
            &pool(),
            &GenericAllocationPolicy,
        );
        assert!(!report.bootstrapped);
        assert_eq!(
            report.stable_cell.status,
            if initialized {
                crate::testing::DiagnosticStableCellStatus::Readable
            } else {
                crate::testing::DiagnosticStableCellStatus::Empty
            }
        );
        assert!(matches!(report.validation, DiagnosticCheck::Passed));
        assert!(report.commit_recovery.is_some());
        assert!(report.ledger.is_none());
        assert_eq!(*backing.borrow(), before);
    }

    // A present invalid physical slot must never be treated as empty storage.
    let mut record = runtime.ledger_record_from_memory().unwrap();
    let genesis = crate::AllocationLedger::new(0, Vec::new()).unwrap();
    record.write_corrupt_ledger(&genesis).unwrap();
    let _cell = Cell::new(
        runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID).unwrap(),
        crate::stable_cell::EncodedLedgerRecord::new(&record).unwrap(),
    );
    let before = backing.borrow().clone();
    let report =
        crate::testing::doctor_report(&runtime, &declarations, &pool(), &GenericAllocationPolicy);
    assert!(matches!(
        report.stable_cell.status,
        crate::testing::DiagnosticStableCellStatus::Readable
    ));
    assert!(matches!(
        report.validation,
        DiagnosticCheck::NotRun {
            code: DiagnosticCode::LedgerRecovery,
            ..
        }
    ));
    assert!(report.ledger.is_none());
    assert!(!runtime.is_bootstrapped());
    assert!(report.commit_recovery.is_some());
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn validation_diagnostic_preserves_unsupported_format_code() {
    let recovered = Err(LedgerCommitError::Recovery(
        crate::CommitRecoveryError::UnsupportedFormat { marker: u64::MAX },
    ));

    let failure = diagnostic_validation_ledger(Some(&recovered))
        .expect_err("unsupported format must block validation");

    assert_eq!(failure.code, DiagnosticCode::UnsupportedFormat);
}

#[test]
fn doctor_binding_agrees_with_warm_bootstrap_for_exact_declarations() {
    let mut runtime = empty_runtime();
    let source = declarations();
    runtime
        .bootstrap(&source, &pool(), &GenericAllocationPolicy)
        .unwrap();
    let before = runtime.growth.backing.borrow().clone();
    for (owner, key) in [
        ("runtime_tests", "runtime_tests.rows.v1"),
        ("different", "runtime_tests.rows.v1"),
        ("runtime_tests", "runtime_tests.changed.v1"),
    ] {
        let requests =
            SealedDeclarationSnapshot::new(vec![crate::MemoryRequest::new(owner, key).unwrap()])
                .unwrap();
        let report =
            crate::testing::doctor_report(&runtime, &requests, &pool(), &GenericAllocationPolicy);
        let warm = runtime.bootstrap(&requests, &pool(), &GenericAllocationPolicy);
        if requests == source {
            assert!(matches!(report.bootstrap_binding, DiagnosticCheck::Passed));
            warm.unwrap();
        } else {
            assert!(matches!(
                report.bootstrap_binding,
                DiagnosticCheck::Failed {
                    code: DiagnosticCode::RuntimeBinding,
                    ..
                }
            ));
            assert!(matches!(
                warm,
                Err(crate::runtime::RuntimeBootstrapError::DeclarationSnapshotMismatch)
            ));
        }
        assert_eq!(*runtime.growth.backing.borrow(), before);
    }
}
