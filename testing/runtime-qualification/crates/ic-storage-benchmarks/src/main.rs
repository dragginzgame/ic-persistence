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
type Samples = BTreeMap<(Case, u32, u32, bool, String, String), Vec<u64>>;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let url = std::env::var("IC_TESTKIT_POCKET_IC_URL")?;
    let wasm = [fs::read(&args.owned_wasm)?, fs::read(&args.dfinity_wasm)?];
    fs::create_dir(&args.output)?;
    fs::write(
        args.output.join("configuration.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "rows": args.rows, "samples": args.samples, "cases": CASES.map(|case| serde_json::json!({"case":case, "key_type":case.key_type(), "value_type":case.value_type(), "logical_value_bytes":case.payloads()})), "batch_sizes": [1,32],
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
                            for start in (0..args.rows).step_by(128) {
                                let setup = Setup {
                                    start: u64::from(start),
                                    count: (args.rows - start).min(128),
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
                            let mut model: BTreeMap<_, _> = (0..u64::from(args.rows))
                                .map(|key| (key, case.expected_value(key, payload, false)))
                                .collect();
                            for operation in OPERATIONS {
                                let request = Request {
                                    case,
                                    operation,
                                    rows: args.rows,
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
                                    .push(actual.instructions);
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
        "rows,case,key_type,value_type,logical_value_bytes,batch_or_scan_limit,pattern,operation,storage_median_instructions,dfinity_median_instructions,storage_over_dfinity"
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
        let a = median(samples);
        let b = median(other);
        writeln!(
            summary,
            "{},{case:?},{},{},{payload},{count},{},{operation},{a},{b},{:.4}",
            args.rows,
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
    let keys = keys(r);
    if r.operation == Operation::Scan {
        return model
            .range(keys[0]..)
            .take(r.count as usize)
            .map(|(&key, value)| (key, Some(value.clone())))
            .collect();
    }
    keys.into_iter()
        .map(|key| {
            let result = match r.operation {
                Operation::Control => None,
                Operation::Get | Operation::Miss => model.get(&key).cloned(),
                Operation::Replace | Operation::Insert => model.insert(
                    key,
                    r.case
                        .expected_value(key, r.payload, r.operation == Operation::Replace),
                ),
                Operation::Remove => model.remove(&key),
                Operation::Scan => unreachable!(),
            };
            (key, result)
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_trace_handles_repeated_hot_updates_and_removals() {
        for case in CASES {
            let payload = case.payloads()[0];
            let mut model: BTreeMap<_, _> = (0..8)
                .map(|key| (key, case.expected_value(key, payload, false)))
                .collect();
            for operation in OPERATIONS {
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
}
