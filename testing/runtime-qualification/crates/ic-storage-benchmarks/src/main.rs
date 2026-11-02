//! Installed, matched map comparison. Testkit owns the explicitly prepared server.
use clap::Parser;
use ic_platform::{pocket_ic::PocketIcBuilder, principal::ExternalPrincipal};
use ic_storage_benchmarks::*;
use std::{collections::BTreeMap, fs, io::Write, path::PathBuf};

#[derive(Parser)]
#[command(about = "Compare original Wasm map workloads on a prepared PocketIC server")]
struct Arguments {
    #[arg(long)]
    owned_wasm: PathBuf,
    #[arg(long)]
    dfinity_wasm: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value_t = 256, value_parser = clap::value_parser!(u32).range(1..=16_384))]
    rows: u32,
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(1..=20))]
    samples: u32,
}
struct Measurement {
    instructions: u64,
    stable_pages: u64,
}
type Samples = BTreeMap<(Case, u32, u32, bool, String, String), Vec<Measurement>>;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let url = std::env::var("IC_TESTKIT_POCKET_IC_URL")?;
    let wasm = [fs::read(&args.owned_wasm)?, fs::read(&args.dfinity_wasm)?];
    fs::create_dir(&args.output)?;
    fs::write(
        args.output.join("configuration.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "rows": args.rows, "structures": ["Map","Set","Vector","Cell","Region"], "samples": args.samples, "cases": CASES.map(|case| serde_json::json!({"case":case, "key_type":case.key_type(), "value_type":case.value_type(), "logical_value_bytes":case.payloads(), "structure":case.structure(), "operations":case.operations(), "population_rows":case.rows(args.rows)})), "batch_sizes": [1,32],
            "patterns": ["uniform", "hot-16"], "seed": "sample index + 1", "population": "ascending keys in batches of 128",
            "backend_order": "alternates each sample", "measurement": "performance_counter(0) around operations; setup, trace/value creation, validation and Wire response excluded",
            "owned_wasm_bytes": wasm[0].len(), "dfinity_wasm_bytes": wasm[1].len(),
            "state": "fresh canister per case/sample/backend; operations run in declared order"
        }))?,
    )?;
    let pic = PocketIcBuilder::new()
        .with_server_url(url.parse()?)
        .with_application_subnet()
        .build();
    let mut raw = fs::File::create(args.output.join("samples.jsonl"))?;
    let mut timings: Samples = BTreeMap::new();
    for case in CASES {
        for &payload in case.payloads() {
            for count in [1, 32] {
                for hot in [false, true] {
                    for sample in 0..args.samples {
                        for backend in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
                            let name = ["ic-storage", "ic-stable-structures"][backend];
                            let canister = pic.create_canister();
                            pic.add_cycles(canister, 100_000_000_000_000);
                            pic.install_canister(
                                canister,
                                wasm[backend].clone(),
                                ic_wire::native::encode_request(0, &case, LIMITS)?,
                                None,
                            );
                            for start in (0..case.rows(args.rows)).step_by(128) {
                                let setup = Setup {
                                    start: u64::from(start),
                                    count: (case.rows(args.rows) - start).min(128),
                                    payload,
                                };
                                let reply = pic
                                    .update_call(
                                        canister,
                                        ExternalPrincipal::anonymous(),
                                        "setup",
                                        ic_wire::native::encode_request(SETUP_ID, &setup, LIMITS)?,
                                    )
                                    .map_err(|e| format!("{name} setup: {e:?}"))?;
                                let length: u64 =
                                    ic_wire::native::decode_response(SETUP_ID, &reply, LIMITS)?;
                                assert_eq!(length, u64::from(start + setup.count));
                            }
                            let mut model: BTreeMap<_, _> = (0..u64::from(case.rows(args.rows)))
                                .map(|key| (key, case.expected_value(key, payload, false)))
                                .collect();
                            for operation in case.operations() {
                                let request = Request {
                                    case,
                                    operation,
                                    rows: case.rows(args.rows),
                                    payload,
                                    count,
                                    seed: u64::from(sample) + 1,
                                    hot,
                                };
                                let reply = pic
                                    .update_call(
                                        canister,
                                        ExternalPrincipal::anonymous(),
                                        "benchmark",
                                        ic_wire::native::encode_request(RUN_ID, &request, LIMITS)?,
                                    )
                                    .map_err(|e| format!("{name} {operation:?}: {e:?}"))?;
                                let actual: Response =
                                    ic_wire::native::decode_response(RUN_ID, &reply, LIMITS)?;
                                writeln!(
                                    raw,
                                    "{}",
                                    serde_json::json!({"backend":name,"sample":sample,"request":request,"response":actual})
                                )?;
                                raw.flush()?;
                                let expected = expected(&mut model, request);
                                assert_eq!(
                                    actual.entries, expected,
                                    "{name} {operation:?}: result mismatch"
                                );
                                assert_eq!(actual.length, model.len() as u64);
                                timings
                                    .entry((
                                        case,
                                        payload,
                                        count,
                                        hot,
                                        format!("{operation:?}"),
                                        name.into(),
                                    ))
                                    .or_default()
                                    .push(Measurement {
                                        instructions: actual.instructions,
                                        stable_pages: actual.stable_pages,
                                    });
                            }
                            let expected_rows: Vec<_> = model.into_iter().collect();
                            for chunk in expected_rows.chunks(128) {
                                let request = Setup {
                                    start: chunk[0].0,
                                    count: chunk.len() as u32,
                                    payload: 0,
                                };
                                let reply = pic
                                    .query_call(
                                        canister,
                                        ExternalPrincipal::anonymous(),
                                        "verify",
                                        ic_wire::native::encode_request(3, &request, LIMITS)?,
                                    )
                                    .map_err(|e| format!("{name} final verification: {e:?}"))?;
                                let rows: Vec<(u64, Vec<u8>)> =
                                    ic_wire::native::decode_response(3, &reply, LIMITS)?;
                                assert_eq!(rows, chunk, "{name}: final map differs");
                            }
                            pic.stop_canister(canister, None).unwrap();
                            pic.delete_canister(canister, None).unwrap();
                        }
                    }
                }
            }
        }
    }
    let mut summary = fs::File::create(args.output.join("summary.csv"))?;
    writeln!(
        summary,
        "rows,structure,case,key_type,value_type,logical_value_bytes,batch_or_scan_limit,pattern,operation,storage_median_instructions,dfinity_median_instructions,storage_over_dfinity,storage_median_stable_pages,dfinity_median_stable_pages"
    )?;
    for ((case, payload, count, hot, operation, backend), samples) in &timings {
        if backend != "ic-storage" {
            continue;
        }
        let other = &timings[&(
            *case,
            *payload,
            *count,
            *hot,
            operation.clone(),
            "ic-stable-structures".into(),
        )];
        let a = median(&samples.iter().map(|s| s.instructions).collect::<Vec<_>>());
        let b = median(&other.iter().map(|s| s.instructions).collect::<Vec<_>>());
        let a_pages = median(&samples.iter().map(|s| s.stable_pages).collect::<Vec<_>>());
        let b_pages = median(&other.iter().map(|s| s.stable_pages).collect::<Vec<_>>());
        writeln!(
            summary,
            "{},{},{case:?},{},{},{payload},{count},{},{operation},{a},{b},{:.4},{a_pages},{b_pages}",
            case.rows(args.rows),
            case.structure(),
            case.key_type(),
            case.value_type(),
            if *hot { "hot-16" } else { "uniform" },
            a as f64 / b as f64
        )?;
    }
    println!(
        "Validated comparison: {}",
        args.output.join("summary.csv").display()
    );
    Ok(())
}
fn median(values: &[u64]) -> u64 {
    let mut values = values.to_vec();
    values.sort_unstable();
    values[values.len() / 2]
}
fn expected(model: &mut BTreeMap<u64, Vec<u8>>, r: Request) -> Vec<(u64, Option<Vec<u8>>)> {
    use Operation::*;
    let ids = keys(r);
    let structure = r.case.structure();
    if matches!(r.operation, First | Last) {
        let entry = if r.operation == First {
            model.first_key_value()
        } else {
            model.last_key_value()
        };
        return entry
            .map(|(&k, v)| vec![(k, Some(v.clone())); r.count as usize])
            .unwrap_or_default();
    }
    if matches!(r.operation, Scan | ReverseScan | KeysScan | BoundedScan) {
        let all = structure == "Set"
            || structure == "Vector"
            || (r.operation == KeysScan && structure == "Map");
        let rows: Vec<_> = model
            .iter()
            .filter(|(k, _)| {
                all || if r.operation == ReverseScan {
                    **k <= ids[0]
                } else {
                    **k >= ids[0]
                        && (r.operation != BoundedScan
                            || **k < ids[0] + u64::from(r.count.div_ceil(2)))
                }
            })
            .map(|(&k, v)| {
                (
                    k,
                    Some(if r.operation == KeysScan || structure == "Set" {
                        vec![]
                    } else {
                        v.clone()
                    }),
                )
            })
            .collect();
        return if r.operation == ReverseScan {
            rows.into_iter().rev().take(r.count as usize).collect()
        } else {
            rows.into_iter().take(r.count as usize).collect()
        };
    }
    ids.into_iter()
        .map(|id| {
            let key = if structure == "Cell" {
                0
            } else if r.operation == Push {
                model.len() as u64
            } else if r.operation == Pop {
                model.len().saturating_sub(1) as u64
            } else {
                id
            };
            let result = match r.operation {
                Control => None,
                Get | Miss => {
                    if structure == "Set" {
                        Some(vec![u8::from(model.contains_key(&key))])
                    } else {
                        model.get(&key).cloned()
                    }
                }
                Contains => Some(vec![u8::from(model.contains_key(&key))]),
                Replace | Insert | Reinsert | Grow | Shrink => {
                    let old = model.insert(
                        key,
                        r.case
                            .expected_value(key, payload_for(r), replacement(r.operation)),
                    );
                    if structure == "Set" {
                        Some(vec![u8::from(old.is_none())])
                    } else {
                        old
                    }
                }
                Remove | RemoveMiss | Pop => {
                    let old = model.remove(&key);
                    if structure == "Set" {
                        Some(vec![u8::from(old.is_some())])
                    } else {
                        old
                    }
                }
                Set => {
                    model.insert(key, r.case.expected_value(key, r.payload, true));
                    None
                }
                Push => {
                    model.insert(key, r.case.expected_value(key, r.payload, false));
                    None
                }
                _ => unreachable!(),
            };
            (key, result)
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
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
            hot: false,
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
                    hot: true,
                };
                let results = expected(&mut model, r);
                if operation == Operation::Miss {
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
                        hot: true,
                    };
                    let entries = expected(&mut model, r);
                    if operation == Operation::Miss {
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
}
