use super::*;
#[test]
fn named_workloads_refuse_mixed_knobs_and_preserve_operation_semantics() {
    let base = [
        "bench",
        "--owned-wasm",
        "owned",
        "--dfinity-wasm",
        "dfinity",
        "--output",
        "report",
    ];
    for name in ["control", "records", "bulk"] {
        let args = Arguments::try_parse_from(base.into_iter().chain(["--workload", name])).unwrap();
        let workload = args.workload.unwrap();
        assert!(
            Arguments::try_parse_from(base.into_iter().chain(["--workload", name, "--rows", "7"]))
                .is_err()
        );
        for &case in workload.cases() {
            for &payload in workload.payloads(case) {
                let rows = case.rows(7);
                let mut model = (0..u64::from(rows))
                    .map(|key| (key, case.expected_value(key, payload, false)))
                    .collect();
                for operation in case.operations() {
                    let request = Request {
                        case,
                        rows,
                        payload,
                        pattern: Pattern::Uniform,
                        count: 1,
                        operation,
                        seed: 1,
                    };
                    if operation == Operation::Grow {
                        assert!(payload_for(request) > payload);
                    }
                    expected(&mut model, request);
                }
                assert_eq!(model.len(), rows as usize);
            }
        }
    }
}
#[test]
fn bucket_selection_is_nonzero_and_survives_initialization_encoding() {
    let base = [
        "bench",
        "--owned-wasm",
        "owned.wasm",
        "--dfinity-wasm",
        "dfinity.wasm",
        "--output",
        "report",
    ];
    assert_eq!(
        Arguments::try_parse_from(base)
            .unwrap()
            .upstream_bucket_pages
            .get(),
        8
    );
    for value in ["0", "65536", "-1"] {
        assert!(
            Arguments::try_parse_from(base.into_iter().chain(["--upstream-bucket-pages", value]))
                .is_err()
        );
    }
    for value in ["1", "8", "128", "65535"] {
        let args =
            Arguments::try_parse_from(base.into_iter().chain(["--upstream-bucket-pages", value]))
                .unwrap();
        let initialization = Initialization {
            case: Case::U64U64,
            cache_bytes: 0,
            upstream_cache_slots: 0,
            upstream_bucket_pages: args.upstream_bucket_pages,
            cold_cache: false,
        };
        let bytes = ic_wire::native::encode_request(0, &initialization, LIMITS).unwrap();
        let decoded: Initialization = ic_wire::native::decode_request(0, &bytes, LIMITS).unwrap();
        assert_eq!(decoded.upstream_bucket_pages, args.upstream_bucket_pages);
    }
    #[derive(serde::Serialize)]
    struct UncheckedInitialization {
        case: Case,
        cache_bytes: usize,
        upstream_cache_slots: usize,
        upstream_bucket_pages: u16,
        cold_cache: bool,
    }
    for pages in [0, 1] {
        let input = UncheckedInitialization {
            case: Case::U64U64,
            cache_bytes: 0,
            upstream_cache_slots: 0,
            upstream_bucket_pages: pages,
            cold_cache: false,
        };
        let bytes = ic_wire::native::encode_request(0, &input, LIMITS).unwrap();
        assert_eq!(
            ic_wire::native::decode_request::<Initialization>(0, &bytes, LIMITS).is_ok(),
            pages != 0
        );
    }
}
#[test]
fn mixed_mutation_rounds_preserve_phase_preconditions_and_population() {
    for case in CASES {
        for &payload in case.payloads() {
            for pattern in [Pattern::Uniform, Pattern::Hot16, Pattern::Zipf] {
                let rows = case.rows(7);
                let mut model: BTreeMap<_, _> = (0..u64::from(rows))
                    .map(|key| (key, case.expected_value(key, payload, false)))
                    .collect();
                for round in 0..3 {
                    for operation in case.operations() {
                        expected(
                            &mut model,
                            Request {
                                case,
                                rows,
                                payload,
                                pattern,
                                operation,
                                count: 32,
                                seed: 1 + round * 7919,
                            },
                        );
                    }
                    assert_eq!(model.len(), rows as usize);
                    assert!(model.keys().copied().eq(0..u64::from(rows)));
                }
            }
        }
    }
}
#[test]
fn mislabeled_removal_and_growth_are_rejected_before_execution() {
    let r = Request {
        case: Case::U64Bytes,
        rows: 7,
        payload: 33,
        pattern: Pattern::Uniform,
        operation: Operation::RemoveMiss,
        count: 1,
        seed: 1,
    };
    let mut model = BTreeMap::from([(keys(r)[0], vec![])]);
    assert!(std::panic::catch_unwind(|| admit_phase(&model, r)).is_err());
    model.clear();
    assert!(
        std::panic::catch_unwind(|| admit_phase(
            &model,
            Request {
                operation: Operation::Grow,
                ..r
            }
        ))
        .is_err()
    );
}
#[test]
fn endpoint_and_bounded_scan_models_admit_empty_and_exclude_upper_bound() {
    let mut model = BTreeMap::from([(0, vec![10]), (1, vec![11]), (2, vec![12])]);
    let mut r = Request {
        case: Case::U64U64,
        operation: Operation::BoundedScan,
        rows: 8,
        payload: 8,
        count: 4,
        seed: 0,
        pattern: Pattern::Uniform,
    };
    assert_eq!(keys(r)[0], 0);
    assert_eq!(
        expected(&mut model, r),
        vec![(0, Some(vec![10])), (1, Some(vec![11]))]
    );
    r.operation = Operation::First;
    assert_eq!(expected(&mut model, r), vec![(0, Some(vec![10])); 4]);
    r.operation = Operation::Last;
    assert_eq!(expected(&mut model, r), vec![(2, Some(vec![12])); 4]);
    model.clear();
    assert!(expected(&mut model, r).is_empty());
    r.operation = Operation::First;
    assert!(expected(&mut model, r).is_empty());
}
#[test]
fn reference_trace_handles_repeated_hot_updates_and_removals() {
    for case in CASES.into_iter().filter(|c| c.structure() == "Map") {
        let payload = case.payloads()[0];
        let mut model: BTreeMap<_, _> = (0..8)
            .map(|key| (key, case.expected_value(key, payload, false)))
            .collect();
        for operation in case.operations() {
            let r = Request {
                case,
                operation,
                rows: 8,
                payload,
                count: 32,
                seed: 1,
                pattern: Pattern::Hot16,
            };
            let results = expected(&mut model, r);
            if operation == Operation::MissAbove {
                assert!(results.iter().all(|(_, value)| value.is_none()));
            }
            if operation == Operation::Replace {
                assert!(results.iter().all(|(_, value)| value.is_some()));
            }
            if operation == Operation::Remove {
                assert!(results.iter().any(|(_, value)| value.is_none()));
            }
        }
    }
    assert_eq!(median(&[30, 10, 20]), 20);
}
#[test]
fn structure_models_cover_cells_sets_vectors_and_regions() {
    for case in CASES {
        let rows = case.rows(7);
        for &payload in case.payloads() {
            let mut model: BTreeMap<_, _> = (0..u64::from(rows))
                .map(|k| (k, case.expected_value(k, payload, false)))
                .collect();
            for operation in case.operations() {
                let r = Request {
                    case,
                    rows,
                    payload,
                    operation,
                    count: 32,
                    seed: 1,
                    pattern: Pattern::Hot16,
                };
                let entries = expected(&mut model, r);
                if operation == Operation::MissAbove {
                    assert!(entries.iter().all(|(_, v)| if case.structure() == "Set" {
                        *v == Some(vec![0])
                    } else {
                        v.is_none()
                    }));
                }
                if operation == Operation::Shrink {
                    assert!(keys(r).iter().all(|key| model[key].is_empty()));
                }
                if operation == Operation::Pop {
                    assert_eq!(model.len(), rows as usize);
                }
                if case.structure() == "Cell" {
                    assert_eq!(model.len(), 1);
                }
            }
        }
    }
}
