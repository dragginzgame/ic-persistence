//! Run the canister probe against an explicitly supplied, pre-admitted server.
mod fixtures;
use ic_testkit::pic::{CandidCallExt, PocketIcBuilder, PocketIcStartupConfig, update_candid_args};
use std::{error::Error, path::Path, time::Duration};
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: ic-storage-bench POCKET_IC_BIN BENCH_WASM".into());
    }
    let wasm = ic_host_fs::read::read_file(Path::new(&args[1]), 32 * 1024 * 1024)?;
    eprintln!(
        "wasm: {:?}",
        ic_host_fs::read::hash_file(Path::new(&args[1]), 32 * 1024 * 1024)?
    );
    eprintln!(
        "server: {:?}",
        ic_host_fs::read::hash_file(Path::new(&args[0]), 512 * 1024 * 1024)?
    );
    let server =
        PocketIcStartupConfig::spawn(&args[0], Duration::from_secs(30)).start_managed_server()?;
    println!("run,engine,profile,operation,instructions,heap_bytes,stable_bytes,result_bytes");
    for run in 0..2 {
        for (profile, fixture) in fixtures::FIXTURES.iter().enumerate() {
            let profile = u8::try_from(profile)?;
            let name = fixture.name;
            let batch = (524_288 / (fixture.key_len + fixture.value_len)).clamp(1, 32) as u64;
            assert!(fixture.count <= 2048 * batch && fixture.kind <= 2);
            for engine in 0u8..=1 {
                let pic = PocketIcBuilder::new()
                    .with_server_url(server.url().parse()?)
                    .with_application_subnet()
                    .build();
                let id = pic.create_canister();
                pic.add_cycles(id, 100_000_000_000_000);
                pic.install_canister(
                    id,
                    wasm.clone(),
                    candid::encode_args((engine, profile))?,
                    None,
                );
                let mut seeded = false;
                for _ in 0..2048 {
                    if pic.update_candid::<bool, _>(id, "seed", (batch,))? {
                        seeded = true;
                        break;
                    }
                }
                if !seeded {
                    return Err("seeding exceeded the bounded call budget".into());
                }
                for (operation, label) in [
                    (0u8, "get"),
                    (4, "missing_get"),
                    (1, "replace"),
                    (2, "scan_keys"),
                    (5, "scan_values"),
                    (6, "shrink"),
                    (7, "regrow"),
                    (3, "remove"),
                    (8, "refill"),
                    (9, "insert_new"),
                ] {
                    let (instructions, heap, stable, result): (u128, u128, u128, u64) =
                        update_candid_args(
                            &pic,
                            id,
                            candid::Principal::anonymous(),
                            "measure",
                            (operation,),
                        )?;
                    println!(
                        "{run},{engine},{name},{label},{instructions},{heap},{stable},{result}"
                    );
                }
            }
        }
    }
    Ok(())
}
