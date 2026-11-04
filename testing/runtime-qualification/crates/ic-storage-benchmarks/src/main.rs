//! Installed, matched map comparison. Testkit owns the explicitly prepared server.
use clap::Parser;
use ic_platform::{pocket_ic::PocketIcBuilder, principal::ExternalPrincipal};
use ic_storage_benchmarks::workloads::{Workload, transfer_rows};
use ic_storage_benchmarks::*;
use std::{collections::BTreeMap, fs, io::Write, path::PathBuf};

#[derive(Parser)]
#[command(about = "Compare original Wasm map workloads on a prepared PocketIC server")]
struct Arguments {
    /// Fixed consumer workload; owns cases, sizes, population, batching and access patterns.
    #[arg(long, value_enum, conflicts_with_all = ["rows", "cases", "patterns", "shuffled", "cache_bytes", "upstream_cache_slots", "cold_cache", "rounds"])]
    workload: Option<Workload>,
    #[arg(long)]
    owned_wasm: PathBuf,
    #[arg(long)]
    dfinity_wasm: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value_t = 256, value_parser = clap::value_parser!(u32).range(1..=1_000_000))]
    rows: u32,
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(1..=20))]
    samples: u32,
    /// Optional concrete case names, e.g. U64U64,VectorU64.
    #[arg(long, value_delimiter = ',')]
    cases: Vec<String>,
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "uniform,hot16"
    )]
    patterns: Vec<Pattern>,
    #[arg(long)]
    shuffled: bool,
    #[arg(long, default_value_t = 0)]
    cache_bytes: usize,
    #[arg(long, default_value_t = 0)]
    upstream_cache_slots: usize,
    /// ICSS MemoryManager bucket size, in 64 KiB pages; does not affect Storage.
    #[arg(long, default_value = "8")]
    upstream_bucket_pages: std::num::NonZeroU16,
    #[arg(long)]
    cold_cache: bool,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=20))]
    rounds: u32,
    #[arg(long, default_value = "0.7.2")]
    reference_revision: String,
}
struct Measurement {
    instructions: u64,
    stable_pages: u64,
}
type Samples = BTreeMap<(Case, u32, u32, Pattern, String, String), Vec<Measurement>>;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::parse();
    if let Some(workload) = args.workload {
        args.rows = workload.rows();
        args.cases = workload
            .cases()
            .iter()
            .map(|case| format!("{case:?}"))
            .collect();
        args.patterns = workload.patterns().to_vec();
        args.shuffled = matches!(workload, Workload::Records);
        args.rounds = workload.rounds();
    }
    let batches: &[u32] = args.workload.map_or(&[1, 32], Workload::counts);
    let payloads = |case: Case| {
        args.workload
            .map_or_else(|| case.payloads(), |workload| workload.payloads(case))
    };
    let cases: Vec<_> = CASES
        .into_iter()
        .chain(
            args.cases
                .iter()
                .any(|name| name == "VectorBytes")
                .then_some(Case::VectorBytes),
        )
        .filter(|case| args.cases.is_empty() || args.cases.contains(&format!("{case:?}")))
        .collect();
    assert!(
        !cases.is_empty()
            && args
                .cases
                .iter()
                .all(|name| cases.iter().any(|case| name == &format!("{case:?}"))),
        "unknown or empty case selection"
    );
    let url = std::env::var("IC_TESTKIT_POCKET_IC_URL")?;
    let wasm = [fs::read(&args.owned_wasm)?, fs::read(&args.dfinity_wasm)?];
    fs::create_dir(&args.output)?;
    fs::write(
        args.output.join("configuration.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "rows": args.rows, "structures": ["Map","Set","Vector","Cell","Region"], "samples": args.samples, "cases": cases.iter().map(|case| serde_json::json!({"case":case, "key_type":case.key_type(), "value_type":case.value_type(), "logical_value_bytes":payloads(*case), "structure":case.structure(), "operations":case.operations(), "population_rows":case.rows(args.rows)})).collect::<Vec<_>>(), "batch_sizes": batches, "workload": args.workload,
            "patterns": args.patterns, "seed": "sample + 1 + round * 7919", "shuffled": args.shuffled,
            "cache_bytes": args.cache_bytes, "upstream_cache_slots": args.upstream_cache_slots, "cold_cache": args.cold_cache, "cache_scope": "ordinary Map only", "rounds": args.rounds, "reference_revision": args.reference_revision,
            "upstream_bucket_pages": args.upstream_bucket_pages,
            "backend_order": "alternates each sample", "measurement": "performance_counter(0) around operations; setup, trace/value creation, validation and Wire response excluded",
            "owned_wasm_bytes": wasm[0].len(), "dfinity_wasm_bytes": wasm[1].len(),
            "state": "fresh canister per case/sample/backend; operations run in declared order"
        }))?,
    )?;
    let pic = PocketIcBuilder::new()
        .with_server_url(url.parse()?)
        .with_application_subnet()
        .build()?;
    let mut raw = fs::File::create(args.output.join("samples.jsonl"))?;
    let mut timings: Samples = BTreeMap::new();
    for case in cases {
        for &payload in payloads(case) {
            for &count in batches {
                for &pattern in &args.patterns {
                    for sample in 0..args.samples {
                        for backend in if sample % 2 == 0 { [0, 1] } else { [1, 0] } {
                            let name = ["ic-storage", "ic-stable-structures"][backend];
                            let canister = pic.create_canister()?;
                            pic.add_cycles(canister, 100_000_000_000_000)?;
                            pic.install_canister(
                                canister,
                                wasm[backend].clone(),
                                ic_wire::native::encode_request(
                                    0,
                                    &Initialization {
                                        case,
                                        cache_bytes: args.cache_bytes,
                                        upstream_cache_slots: args.upstream_cache_slots,
                                        upstream_bucket_pages: args.upstream_bucket_pages,
                                        cold_cache: args.cold_cache,
                                    },
                                    LIMITS,
                                )?,
                                None,
                            )?;
                            let ids = population(
                                case.rows(args.rows),
                                args.shuffled
                                    && matches!(case.structure(), "Map" | "Set" | "Region"),
                                u64::from(sample) + 1,
                            );
                            for start in
                                (0..case.rows(args.rows)).step_by(transfer_rows(payload as usize))
                            {
                                let count = (case.rows(args.rows) - start)
                                    .min(transfer_rows(payload as usize) as u32);
                                let setup = Setup {
                                    start: u64::from(start),
                                    count,
                                    payload,
                                    ids: ids[start as usize..(start + count) as usize].to_vec(),
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
                            for round in 0..args.rounds {
                                for operation in case.operations() {
                                    let request = Request {
                                        case,
                                        operation,
                                        rows: case.rows(args.rows),
                                        payload,
                                        count,
                                        seed: u64::from(sample) + 1 + u64::from(round) * 7919,
                                        pattern,
                                    };
                                    // Admit the intended phase before executing either backend.
                                    let expected = expected(&mut model, request);
                                    let reply = pic
                                        .update_call(
                                            canister,
                                            ExternalPrincipal::anonymous(),
                                            "benchmark",
                                            ic_wire::native::encode_request(
                                                RUN_ID, &request, LIMITS,
                                            )?,
                                        )
                                        .map_err(|e| format!("{name} {operation:?}: {e:?}"))?;
                                    let actual: Response =
                                        ic_wire::native::decode_response(RUN_ID, &reply, LIMITS)?;
                                    writeln!(
                                        raw,
                                        "{}",
                                        serde_json::json!({"backend":name,"sample":sample,"round":round,"request":request,"response":actual})
                                    )?;
                                    raw.flush()?;
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
                                            pattern,
                                            format!("{operation:?}"),
                                            name.into(),
                                        ))
                                        .or_default()
                                        .push(Measurement {
                                            instructions: actual.instructions,
                                            stable_pages: actual.stable_pages,
                                        });
                                }
                            }
                            let expected_rows: Vec<_> = model.into_iter().collect();
                            let largest = expected_rows
                                .iter()
                                .map(|(_, bytes)| bytes.len())
                                .max()
                                .unwrap_or(0);
                            for chunk in expected_rows.chunks(transfer_rows(largest)) {
                                let request = Setup {
                                    start: chunk[0].0,
                                    count: chunk.len() as u32,
                                    payload: 0,
                                    ids: Vec::new(),
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
        "rows,structure,case,key_type,value_type,logical_value_bytes,batch_or_scan_limit,pattern,operation,storage_median_instructions,dfinity_median_instructions,storage_over_dfinity,storage_median_stable_pages,dfinity_median_stable_pages,upstream_bucket_pages"
    )?;
    for ((case, payload, count, pattern, operation, backend), samples) in &timings {
        if backend != "ic-storage" {
            continue;
        }
        let other = &timings[&(
            *case,
            *payload,
            *count,
            *pattern,
            operation.clone(),
            "ic-stable-structures".into(),
        )];
        let a = median(&samples.iter().map(|s| s.instructions).collect::<Vec<_>>());
        let b = median(&other.iter().map(|s| s.instructions).collect::<Vec<_>>());
        let a_pages = median(&samples.iter().map(|s| s.stable_pages).collect::<Vec<_>>());
        let b_pages = median(&other.iter().map(|s| s.stable_pages).collect::<Vec<_>>());
        writeln!(
            summary,
            "{},{},{case:?},{},{},{payload},{count},{pattern:?},{operation},{a},{b},{:.4},{a_pages},{b_pages},{}",
            case.rows(args.rows),
            case.structure(),
            case.key_type(),
            case.value_type(),
            a as f64 / b as f64,
            args.upstream_bucket_pages
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
fn population(rows: u32, shuffled: bool, mut state: u64) -> Vec<u64> {
    let mut ids: Vec<_> = (0..u64::from(rows)).collect();
    if shuffled {
        for end in (1..ids.len()).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let slot = usize::try_from(state % (end as u64 + 1)).unwrap();
            ids.swap(end, slot);
        }
    }
    ids
}
fn expected(model: &mut BTreeMap<u64, Vec<u8>>, r: Request) -> Vec<(u64, Option<Vec<u8>>)> {
    use Operation::*;
    admit_phase(model, r);
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
                Get | MissAbove => {
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
                Remove | RemoveMiss | RemoveInserted | Pop => {
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
// Preconditions apply at phase entry. Repeated hot keys can legitimately turn
// later removals/reinsertions in the same batch into misses/replacements.
fn admit_phase(model: &BTreeMap<u64, Vec<u8>>, r: Request) {
    use Operation::*;
    for key in keys(r) {
        match r.operation {
            MissAbove | RemoveMiss | Insert | Reinsert => {
                assert!(
                    !model.contains_key(&key),
                    "{:?}: key {key} must be absent",
                    r.operation
                );
            }
            Get | Contains | Replace | Set | Remove | RemoveInserted | Grow | Shrink => {
                let old = model
                    .get(&key)
                    .unwrap_or_else(|| panic!("{:?}: key {key} must exist", r.operation));
                if matches!(r.operation, Grow | Shrink) {
                    let previous_payload = if r.operation == Grow {
                        r.payload
                    } else {
                        payload_for(Request {
                            operation: Grow,
                            ..r
                        })
                    };
                    assert_eq!(
                        *old,
                        r.case.expected_value(key, previous_payload, true),
                        "{:?}: unexpected previous value",
                        r.operation
                    );
                }
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    // The library owns tests.rs; keep binary test bodies in the shared tests directory.
    use super::*;
    mod driver;
}
