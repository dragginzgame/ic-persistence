//! Explicit native allocation measurements; not IC instruction benchmarks.
use crate::{
    AllocationLedger, GenericAllocationPolicy, LedgerCommitError, LedgerCommitStore,
    MemoryManagerConfig, MemoryRuntime, SealedDeclarationSnapshot,
    stable_cell::EncodedLedgerRecord,
};
use ic_platform::stable_structures::{
    Memory, Storable, VectorMemory,
    memory_manager::{MemoryId, MemoryManager},
};
use ic_testkit_native::{MeasuredAllocator, measure as measure_allocations};
use std::{alloc::System, hint::black_box};

#[global_allocator]
static ALLOCATOR: MeasuredAllocator = MeasuredAllocator::new(System);

fn measure(phase: &str, generations: u64, iterations: usize, operation: impl FnMut()) {
    let counts =
        measure_allocations(iterations, operation).expect("non-nested allocation workload");
    let elapsed = counts.elapsed.as_micros();
    assert_eq!(
        counts.net_bytes, 0,
        "all measured allocations must be released"
    );
    println!(
        "{phase},{generations},{iterations},{},{},{},{elapsed}",
        counts.allocations, counts.reallocations, counts.peak_bytes
    );
}

#[test]
#[ignore = "explicit matched allocation measurement, run in release mode with one test thread"]
fn ledger_encoding_and_validation_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    for count in [0, 1, 64, 255] {
        for generations in [1, 1024, u64::MAX] {
            let records = (0..count)
                .map(|id| {
                    crate::AllocationRecord::from_declaration(
                        &crate::AllocationDeclaration::memory_manager(
                            format!("app.store{id}.v1"),
                            u8::try_from(id).unwrap(),
                        )
                        .unwrap(),
                    )
                })
                .collect();
            let ledger = AllocationLedger::new(generations, records).unwrap();
            let mut store = LedgerCommitStore::default();
            store.commit(&ledger).unwrap();
            let record = store;
            measure(&format!("encode_records_{count}"), generations, 10, || {
                drop(black_box(
                    EncodedLedgerRecord::new(&record).unwrap().into_bytes(),
                ));
            });
            measure(
                &format!("validate_records_{count}"),
                generations,
                10,
                || {
                    black_box(ledger.validate_integrity()).unwrap();
                },
            );
            measure(&format!("recover_records_{count}"), generations, 10, || {
                drop(black_box(record.recover().unwrap()));
            });
            measure(&format!("commit_records_{count}"), generations, 10, || {
                let mut store = LedgerCommitStore::default();
                drop(black_box(store.commit(&ledger).unwrap()));
            });
        }
    }
}
// Only the stable key differs from a valid current ledger. The physical store
// owns a correctly checksummed opaque envelope; text must be checked by logical
// recovery, not by decoding the enclosing physical DTO.
fn key_text_store(text_bytes: usize) -> LedgerCommitStore {
    let declaration = crate::AllocationDeclaration::memory_manager("app.rows.v1", 100).unwrap();
    let ledger = AllocationLedger::new(
        1,
        vec![crate::AllocationRecord::from_declaration(&declaration)],
    )
    .unwrap();
    let payload = crate::test_wire::to_vec(&vec![(
        format!("app.{}.v1", "x".repeat(text_bytes - 7)),
        ledger.records()[0].slot().id(),
    )])
    .unwrap();
    let mut physical = crate::LedgerCommitStore::default();
    physical.commit_payload_at_generation(1, payload).unwrap();
    physical
}

#[test]
#[ignore = "explicit matched text recovery allocation measurement, run in release mode with one test thread"]
fn ledger_text_recovery_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    for text_bytes in [128, 129, 1024, 8192, 32 * 1024] {
        let store = key_text_store(text_bytes);
        let phase = format!("recover_text_{text_bytes}");
        measure(&phase, 1, 10, || {
            let result = black_box(store.recover());
            if text_bytes <= 128 {
                assert_eq!(result.unwrap().current_generation(), 1);
            } else {
                assert!(matches!(result, Err(LedgerCommitError::Codec(_))));
            }
        });
    }
}

#[test]
fn text_measurement_fixture_changes_only_key_bound() {
    let valid = key_text_store(128).recover().unwrap();
    assert_eq!(valid.current_generation(), 1);
    let record = &valid.ledger().records()[0];
    assert_eq!(record.stable_key().as_str().len(), 128);
    assert_eq!(record.slot().id(), 100);
    assert!(matches!(
        key_text_store(129).recover(),
        Err(LedgerCommitError::Codec(_))
    ));
}

#[test]
#[ignore = "explicit matched diagnostics allocation measurement, run in release mode with one test thread"]
fn allocation_diagnostics_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    let declarations = SealedDeclarationSnapshot::new(vec![]).unwrap();
    for populated in [false, true] {
        let backing = VectorMemory::default();
        if populated {
            let manager = MemoryManager::init_with_bucket_size(backing.clone(), 1);
            for id in [100, 254] {
                assert_eq!(manager.get(MemoryId::new(id)).grow(2), 0);
            }
        }
        let mut runtime =
            MemoryRuntime::new_with_config(backing, MemoryManagerConfig::new(1).unwrap()).unwrap();
        let prefix = if populated { "populated" } else { "empty" };
        assert_eq!(
            runtime
                .memory_allocation_summary()
                .unwrap()
                .allocated_buckets,
            if populated { 4 } else { 0 }
        );
        for bootstrapped in [false, true] {
            if bootstrapped {
                runtime
                    .bootstrap(&declarations, &pool(), &GenericAllocationPolicy)
                    .unwrap();
            }
            let generation = u64::from(bootstrapped);
            let current_generation = bootstrapped.then_some(generation);
            let state = if bootstrapped {
                "bootstrapped"
            } else {
                "unbootstrapped"
            };
            let summary_phase = format!("{prefix}_{state}_summary");
            measure(&summary_phase, generation, 100, || {
                let summary = black_box(runtime.memory_allocation_summary().unwrap());
                assert_eq!(summary.current_generation, current_generation);
            });
            let report_phase = format!("{prefix}_{state}_report");
            measure(&report_phase, generation, 100, || {
                let report = black_box(runtime.memory_allocations().unwrap());
                assert_eq!(report.memories.len(), 255);
                assert_eq!(report.summary.current_generation, current_generation);
            });
        }
    }
}

fn pool() -> crate::MemoryAllocationPool {
    // The populated measurement deliberately owns two fixed unmanaged IDs.
    // Admit that physical custody explicitly before runtime bootstrap.
    crate::MemoryAllocationPool::new(vec![], (100..=100).chain(254..=254).collect()).unwrap()
}

#[test]
#[ignore = "explicit matched declaration resolution allocation measurement, run in release mode with one test thread"]
fn declaration_resolution_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    let ledger = crate::RecoveredLedger::from_trusted_ledger(AllocationLedger::empty_genesis());
    for count in [1, 32, 245] {
        let owner = "a".repeat(256);
        let requests = (0..count)
            .map(|id| {
                crate::MemoryRequest::new(owner.clone(), &format!("app.store{id:03}.v1")).unwrap()
            })
            .collect();
        let declarations = SealedDeclarationSnapshot::new(requests).unwrap();
        let pool = crate::MemoryAllocationPool::new(
            vec![crate::MemoryAuthority::new(owner, "app.").unwrap()],
            vec![],
        )
        .unwrap();
        measure(&format!("resolve_{count}"), 0, 10, || {
            let resolved = black_box(
                declarations
                    .resolve(&ledger, &ledger.ledger().occupied_slots(), vec![], &pool)
                    .unwrap(),
            );
            assert_eq!(resolved.declarations().len(), count + 1);
        });
        measure(&format!("fresh_bootstrap_{count}"), 0, 10, || {
            let mut runtime = MemoryRuntime::new_with_config(
                VectorMemory::default(),
                MemoryManagerConfig::new(1).unwrap(),
            )
            .unwrap();
            assert_eq!(
                runtime
                    .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
                    .unwrap(),
                1
            );
        });
    }
}

#[test]
#[ignore = "explicit matched doctor allocation measurement; release mode and one thread"]
fn doctor_report_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    let pool = crate::MemoryAllocationPool::new(
        vec![crate::MemoryAuthority::new("doctor", "app.").unwrap()],
        vec![],
    )
    .unwrap();
    for count in [0, 1, 64, 245] {
        let requests = (0..count)
            .map(|id| {
                let prefix = format!("app.store{id:03}.");
                let key = format!("{prefix}{}.v1", "k".repeat(128 - prefix.len() - 3));
                crate::MemoryRequest::new("doctor", &key).unwrap()
            })
            .collect();
        let declarations = SealedDeclarationSnapshot::new(requests).unwrap();
        let mut runtime = MemoryRuntime::new_with_config(
            VectorMemory::default(),
            MemoryManagerConfig::new(1).unwrap(),
        )
        .unwrap();
        runtime
            .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
            .unwrap();
        measure(&format!("doctor_{count}"), 1, 20, || {
            let report = black_box(crate::testing::doctor_report(
                &runtime,
                &declarations,
                &pool,
                &GenericAllocationPolicy,
            ));
            assert_eq!(report.ledger.as_ref().unwrap().records.len(), count + 1);
            assert_eq!(report.validation, crate::testing::DiagnosticCheck::Passed);
        });
    }
}

#[test]
#[ignore = "explicit matched steady-state snapshot measurement; release mode and one thread"]
fn steady_state_snapshot_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    for (count, long) in [
        (0, false),
        (1, false),
        (64, false),
        (255, false),
        (255, true),
    ] {
        let records = (0..count)
            .map(|id| {
                let key = if long {
                    format!("app.store{id:03}.{}.v1", "x".repeat(112))
                } else {
                    format!("app.store{id:03}.v1")
                };
                crate::AllocationRecord::from_declaration(
                    &crate::AllocationDeclaration::memory_manager(key, u8::try_from(id).unwrap())
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let mut store = LedgerCommitStore::default();
        store
            .commit(&AllocationLedger::new(1, records.clone()).unwrap())
            .unwrap();
        store
            .commit(&AllocationLedger::new(2, records).unwrap())
            .unwrap();
        let record = store;
        let label = format!(
            "snapshot_{count}_{}",
            if long { "maximum" } else { "short" }
        );
        println!(
            "encoded_bytes,{label},{}",
            EncodedLedgerRecord::new(&record).unwrap().len()
        );
        measure(&format!("encode_{label}"), 2, 10, || {
            drop(black_box(
                EncodedLedgerRecord::new(&record).unwrap().into_bytes(),
            ));
        });
        let memory = VectorMemory::default();
        drop(ic_platform::stable_structures::Cell::new(
            memory.clone(),
            EncodedLedgerRecord::new(&record).unwrap(),
        ));
        measure(&format!("reopen_{label}"), 2, 10, || {
            let decoded = crate::decode_stable_cell_commit_store_from_memory(&memory).unwrap();
            drop(black_box(decoded.recover().unwrap()));
        });
        measure(&format!("reopen_commit_{label}"), 2, 10, || {
            let mut decoded = crate::decode_stable_cell_commit_store_from_memory(&memory).unwrap();
            let mut next = decoded.recover().unwrap().into_ledger();
            next.current_generation += 1;
            decoded.commit_generation(&next).unwrap();
            drop(ic_platform::stable_structures::Cell::new(
                memory.clone(),
                EncodedLedgerRecord::new(&decoded).unwrap(),
            ));
        });
    }
}
