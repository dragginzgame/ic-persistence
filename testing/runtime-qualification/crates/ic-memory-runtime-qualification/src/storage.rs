//! Installed Storage adapters under the same caller-owned server as Memory.
use ic_platform::pocket_ic::PocketIc;
use std::{env, fs, time::Duration};

pub(super) fn qualify(pic: &PocketIc) {
    let wasm =
        fs::read(env::var("IC_STORAGE_QUALIFICATION_WASM").expect("Storage fixture path")).unwrap();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm.clone(), Vec::new(), None);
    let snapshot = || {
        pic.query_call(canister, canister, "snapshot", vec![])
            .unwrap()
    };
    let expected = |generation: u64, values: [u64; 3]| {
        [generation, values[0], values[1], values[2]]
            .into_iter()
            .flat_map(u64::to_le_bytes)
            .collect::<Vec<_>>()
    };
    let initial = snapshot();
    assert_eq!(initial, expected(1, [10, 20, 30]));
    super::assert_trap(
        pic.update_call(canister, canister, "mutate", vec![1])
            .unwrap_err(),
    );
    assert_eq!(
        snapshot(),
        initial,
        "trapped update rolls back all collection writes"
    );
    pic.update_call(canister, canister, "mutate", vec![0])
        .unwrap();
    let committed = snapshot();
    assert_eq!(committed, expected(1, [100, 101, 102]));
    pic.advance_time(Duration::from_secs(300));
    super::assert_trap(
        pic.upgrade_canister(canister, wasm.clone(), vec![1], None)
            .unwrap_err(),
    );
    assert_eq!(
        snapshot(),
        committed,
        "failed upgrade preserves collections and generation"
    );
    pic.advance_time(Duration::from_secs(300));
    pic.upgrade_canister(canister, wasm, vec![0], None).unwrap();
    assert_eq!(snapshot(), expected(2, [100, 101, 102]));
    println!(
        "installed Storage: map, bounded cell and bounded region scan survive upgrade; update/upgrade traps roll back collection writes"
    );
}
