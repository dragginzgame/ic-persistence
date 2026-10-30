//! Construction contracts.

use super::*;

#[test]
fn construction_initializes_empty_memory_and_accepts_current_manager_memory() {
    let backing = VectorMemory::default();
    let runtime =
        MemoryRuntime::new(backing.clone()).expect("empty backing memory should initialize");
    assert_eq!(&read_one_page(&backing)[..4], b"MGR\x01");

    drop(runtime);
    let recovered =
        MemoryRuntime::new(backing).expect("current MemoryManager backing should be accepted");
    assert!(!recovered.is_bootstrapped());
}

#[test]
fn construction_rejects_foreign_nonempty_memory_without_writing() {
    let (backing, original) = one_page_backing(*b"DATA");

    let Err(error) = MemoryRuntime::new(backing.clone()) else {
        panic!("foreign backing memory must be rejected");
    };

    assert_eq!(
        error,
        RuntimeConstructionError::ForeignMemory {
            observed_magic: *b"DAT",
        }
    );
    assert_eq!(read_one_page(&backing), original);
}

#[test]
fn construction_rejects_caller_pregrown_blank_memory_without_writing() {
    for configured in [false, true] {
        let backing = VectorMemory::default();
        assert_eq!(backing.grow(1), 0);
        let original = read_one_page(&backing);
        let result = if configured {
            MemoryRuntime::new_with_config(
                backing.clone(),
                crate::runtime::MemoryManagerConfig::new(16).unwrap(),
            )
        } else {
            MemoryRuntime::new(backing.clone())
        };
        let Err(error) = result else {
            panic!("caller-pregrown blank memory must remain foreign input");
        };
        assert_eq!(
            error,
            RuntimeConstructionError::ForeignMemory {
                observed_magic: [0; 3]
            }
        );
        assert_eq!(read_one_page(&backing), original);
    }
}

#[test]
fn construction_rejects_unsupported_manager_version_without_writing() {
    let (backing, original) = one_page_backing(*b"MGR\x02");

    let Err(error) = MemoryRuntime::new(backing.clone()) else {
        panic!("unsupported MemoryManager backing must be rejected");
    };

    assert_eq!(
        error,
        RuntimeConstructionError::UnsupportedMemoryManagerVersion {
            observed: 2,
            supported: 1,
        }
    );
    assert_eq!(read_one_page(&backing), original);
}

#[test]
fn separate_runtimes_have_independent_bootstrap_authority_and_memory() {
    let declarations = declarations();
    let mut runtime_a = empty_runtime();
    let mut runtime_b = empty_runtime();
    let policy = CountingPolicy(std::cell::Cell::new(0));

    runtime_a
        .bootstrap(&declarations, &pool(), &policy)
        .expect("runtime A bootstrap");
    assert_eq!(policy.0.get(), 1);
    assert!(runtime_a.is_bootstrapped());
    assert!(!runtime_b.is_bootstrapped());
    assert_eq!(
        runtime_b.committed_allocations().expect_err("runtime B"),
        RuntimeOpenError::NotBootstrapped
    );

    let memory_a = runtime_a
        .open_memory("runtime_tests.rows.v1")
        .expect("runtime A memory");
    memory_a.grow(1).unwrap();
    memory_a.write(0, b"runtime-a");

    runtime_b
        .bootstrap(&declarations, &pool(), &policy)
        .expect("runtime B bootstrap");
    assert_eq!(policy.0.get(), 2);
    let memory_b = runtime_b
        .open_memory("runtime_tests.rows.v1")
        .expect("runtime B memory");
    assert_eq!(memory_b.size(), 0);
    memory_b.grow(1).unwrap();
    let mut bytes = [0; 9];
    memory_b.read(0, &mut bytes);
    assert_eq!(&bytes, &[0; 9]);
    assert_eq!(
        runtime_a
            .diagnostic_export()
            .expect("runtime A diagnostics")
            .current_generation,
        1
    );
    assert_eq!(
        runtime_b
            .diagnostic_export()
            .expect("runtime B diagnostics")
            .current_generation,
        1
    );
}

#[test]
fn concurrent_independent_runtimes_do_not_share_bootstrap_state() {
    let declarations = declarations();
    let first_declarations = declarations.clone();
    let second_declarations = declarations;

    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(move || {
            let mut runtime = empty_runtime();
            let generation = runtime
                .bootstrap(&first_declarations, &pool(), &GenericAllocationPolicy)
                .expect("first bootstrap");
            let diagnostic_generation = runtime
                .diagnostic_export()
                .expect("first diagnostics")
                .current_generation;
            (generation, diagnostic_generation)
        });
        let second = scope.spawn(move || {
            let mut runtime = empty_runtime();
            let generation = runtime
                .bootstrap(&second_declarations, &pool(), &GenericAllocationPolicy)
                .expect("second bootstrap");
            let diagnostic_generation = runtime
                .diagnostic_export()
                .expect("second diagnostics")
                .current_generation;
            (generation, diagnostic_generation)
        });
        (
            first.join().expect("first runtime thread"),
            second.join().expect("second runtime thread"),
        )
    });

    assert_eq!(first, (1, 1));
    assert_eq!(second, (1, 1));
}

#[test]
fn repeated_bootstrap_accepts_independently_sealed_equivalent_declarations() {
    let declarations = declarations();
    let equivalent = SealedDeclarationSnapshot::new(declarations.requests().to_vec())
        .expect("equivalent declarations");
    assert_eq!(equivalent, declarations);
    assert!(!declarations.shares_storage_with(&equivalent));

    let policy = CountingPolicy(std::cell::Cell::new(0));
    let mut runtime = empty_runtime();
    let generation = runtime
        .bootstrap(&declarations, &pool(), &policy)
        .expect("bootstrap");
    assert!(matches!(
        crate::testing::doctor_report(&runtime, &equivalent, &pool(), &policy).bootstrap_binding,
        DiagnosticCheck::Passed
    ));
    let policy_calls = policy.0.get();
    let ledger_before = runtime.ledger_record_from_memory().expect("ledger before");

    assert_eq!(
        runtime
            .bootstrap(&equivalent, &pool(), &policy)
            .expect("equivalent bootstrap"),
        generation
    );
    assert_eq!(policy.0.get(), policy_calls);
    assert_eq!(
        runtime
            .verify_bootstrap(&equivalent, &pool(), &policy)
            .unwrap(),
        generation
    );
    assert_eq!(policy.0.get(), policy_calls);
    assert_eq!(
        runtime.ledger_record_from_memory().expect("ledger after"),
        ledger_before
    );
}
