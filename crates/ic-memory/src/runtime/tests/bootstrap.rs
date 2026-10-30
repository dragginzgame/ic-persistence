//! Bootstrap contracts.

use super::*;

#[test]
fn repeated_bootstrap_is_idempotent_and_existing_memory_recovers() {
    let declarations = declarations();
    let backing = VectorMemory::default();
    let generation = {
        let mut runtime = MemoryRuntime::new(backing.clone()).expect("empty backing memory");
        let first = runtime
            .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
            .expect("first bootstrap");
        let second = runtime
            .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
            .expect("idempotent bootstrap");
        assert_eq!(first, second);
        let memory = runtime
            .open_memory("runtime_tests.rows.v1")
            .expect("first runtime memory");
        memory.grow(1).unwrap();
        memory.write(0, b"persisted");
        first
    };

    let mut recovered_runtime =
        MemoryRuntime::new(backing).expect("existing MemoryManager backing memory");
    let recovered_generation = recovered_runtime
        .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
        .expect("recover existing backing memory");
    assert!(recovered_generation >= generation);
    assert_eq!(
        recovered_runtime
            .diagnostic_export()
            .expect("runtime diagnostic")
            .current_generation,
        recovered_generation
    );
    let memory = recovered_runtime
        .open_memory("runtime_tests.rows.v1")
        .expect("recovered runtime memory");
    let mut bytes = [0; 9];
    memory.read(0, &mut bytes);
    assert_eq!(&bytes, b"persisted");
}

#[test]
fn repeated_bootstrap_is_bound_to_declarations_and_policy_identity() {
    let declarations = declarations();
    let mut runtime = empty_runtime();

    let generation = runtime
        .bootstrap(&declarations, &pool(), &IdentityPolicy::new([1; 32]))
        .expect("first bootstrap");
    let repeated_generation = runtime
        .bootstrap(&declarations, &pool(), &IdentityPolicy::new([1; 32]))
        .expect("same semantic policy identity");
    assert_eq!(repeated_generation, generation);

    let policy_error = runtime
        .bootstrap(&declarations, &pool(), &IdentityPolicy::new([2; 32]))
        .expect_err("changed policy identity");
    assert!(matches!(
        policy_error,
        crate::runtime::RuntimeBootstrapError::PolicyIdentityMismatch { .. }
    ));
    assert!(matches!(
        runtime.verify_bootstrap(&declarations, &pool(), &IdentityPolicy::new([2; 32])),
        Err(crate::runtime::RuntimeBootstrapError::PolicyIdentityMismatch { .. })
    ));

    let alternate_declarations = SealedDeclarationSnapshot::new(vec![
        crate::MemoryRequest::new("alternate_runtime_tests", "alternate_runtime_tests.rows.v1")
            .unwrap(),
    ])
    .unwrap();
    let declaration_error = runtime
        .bootstrap(
            &alternate_declarations,
            &pool(),
            &IdentityPolicy::new([1; 32]),
        )
        .expect_err("changed declaration snapshot");
    assert!(matches!(
        declaration_error,
        crate::runtime::RuntimeBootstrapError::DeclarationSnapshotMismatch
    ));
    assert_eq!(
        runtime
            .diagnostic_export()
            .expect("unchanged ledger")
            .current_generation,
        generation
    );
}

#[test]
fn policy_token_changes_are_runtime_bound() {
    let declarations = declarations();
    let mut runtime = empty_runtime();
    let committed = runtime
        .bootstrap(&declarations, &pool(), &IdentityPolicy::new([0; 32]))
        .unwrap();
    let before = runtime.ledger_record_from_memory().unwrap();
    for index in 0..32 {
        let mut changed = [0; 32];
        changed[index] = 1;
        assert!(matches!(
            runtime.bootstrap(&declarations, &pool(), &IdentityPolicy::new(changed)),
            Err(crate::runtime::RuntimeBootstrapError::PolicyIdentityMismatch { .. })
        ));
        let doctor = crate::testing::doctor_report(
            &runtime,
            &declarations,
            &pool(),
            &IdentityPolicy::new(changed),
        );
        assert_eq!(doctor.tested_policy_identity.as_bytes(), &changed);
        assert!(matches!(
            doctor.bootstrap_binding,
            DiagnosticCheck::Failed {
                code: DiagnosticCode::RuntimeBinding,
                ..
            }
        ));
    }
    assert_eq!(runtime.ledger_record_from_memory().unwrap(), before);
    assert_eq!(
        runtime
            .bootstrap(&declarations, &pool(), &IdentityPolicy::new([0; 32]))
            .unwrap(),
        committed
    );
}

#[test]
fn open_errors_and_failed_bootstrap_do_not_publish_authority() {
    let declarations = declarations();
    let mut runtime = empty_runtime();

    let Err(open_before_bootstrap) = runtime.open_memory("runtime_tests.rows.v1") else {
        panic!("open before bootstrap must fail");
    };
    assert_eq!(open_before_bootstrap, RuntimeOpenError::NotBootstrapped);
    assert!(matches!(
        runtime.bootstrap(&declarations, &pool(), &RejectPolicy),
        Err(crate::runtime::RuntimeBootstrapError::Validation(
            crate::AllocationValidationError::Policy("rejected")
        ))
    ));
    assert!(!runtime.is_bootstrapped());
    let rejected_doctor =
        crate::testing::doctor_report(&runtime, &declarations, &pool(), &RejectPolicy);
    assert!(!rejected_doctor.bootstrapped);
    assert!(matches!(
        rejected_doctor.validation,
        DiagnosticCheck::Failed {
            code: DiagnosticCode::AllocationValidation,
            ..
        }
    ));
    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::NotBootstrapped)
    ));
    assert_eq!(
        runtime.committed_allocations().expect_err("no capability"),
        RuntimeOpenError::NotBootstrapped
    );

    runtime
        .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
        .expect("successful retry");
    let Err(wrong_key) = runtime.open_memory("runtime_tests.missing.v1") else {
        panic!("wrong key must fail");
    };
    assert!(matches!(
        wrong_key,
        RuntimeOpenError::StableKeyNotCommitted(_)
    ));
}

#[test]
fn pool_admission_precedes_application_validation_and_preserves_retry() {
    let source = declarations();
    let backing = VectorMemory::default();
    let mut original = MemoryRuntime::new(backing.clone()).unwrap();
    original
        .bootstrap(&source, &pool(), &GenericAllocationPolicy)
        .unwrap();
    drop(original);
    let before = backing.borrow().clone();
    let foreign = SealedDeclarationSnapshot::new(vec![
        crate::MemoryRequest::new("foreign", "runtime_tests.rows.v1").unwrap(),
    ])
    .unwrap();
    let excluded =
        crate::MemoryAllocationPool::new(pool().authorities().to_vec(), (10..=254).collect())
            .unwrap();
    let policy = CountingPolicy(std::cell::Cell::new(0));
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    for (requests, host_pool) in [(&foreign, pool()), (&source, excluded)] {
        assert!(matches!(
            crate::testing::doctor_report(&runtime, requests, &host_pool, &policy).validation,
            DiagnosticCheck::Failed {
                code: DiagnosticCode::AllocationValidation,
                ..
            }
        ));
        assert!(matches!(
            runtime.bootstrap(requests, &host_pool, &policy),
            Err(crate::runtime::RuntimeBootstrapError::Resolution(
                crate::runtime::MemoryResolutionError::Pool(_)
            ))
        ));
        assert_eq!(policy.0.get(), 0);
        assert_eq!(*backing.borrow(), before);
        assert!(!runtime.is_bootstrapped());
    }
    runtime.bootstrap(&source, &pool(), &policy).unwrap();
    assert_eq!(policy.0.get(), 1); // One external allocation; governance stays private.
    assert_eq!(runtime.memory_id("runtime_tests.rows.v1"), Ok(120));
}
