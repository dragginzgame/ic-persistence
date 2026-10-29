//! Installed Storage adapters under the same caller-owned server as Memory.
use ic_platform::pocket_ic::PocketIc;
use std::{env, fs, time::Duration};

pub(super) fn qualify(pic: &PocketIc) {
    let wasm =
        fs::read(env::var("IC_STORAGE_QUALIFICATION_WASM").expect("Storage fixture path")).unwrap();
    let canister = super::install_fixture(pic, &wasm);
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
    for mode in 0..4 {
        pic.update_call(canister, canister, "cell_load_failure", vec![mode])
            .unwrap();
        assert_eq!(
            snapshot(),
            initial,
            "rejected cell load preserves live collections"
        );
    }
    super::assert_trap(
        pic.update_call(canister, canister, "cell_load_failure", vec![4])
            .unwrap_err(),
    );
    assert_eq!(
        snapshot(),
        initial,
        "cell decoder trap preserves live collections"
    );
    // The failed call's lease is rolled back with the Wasm heap and can reopen.
    pic.update_call(canister, canister, "cell_load_failure", vec![3])
        .unwrap();
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
        "installed Storage: non-initializing cell admission preserves bytes/pages and leases; cells, map and region survive upgrade; decoder/update/upgrade traps roll back"
    );
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires an explicitly prepared server URL and Storage fixture Wasm"]
    fn installed_storage_adapters() {
        let pic = crate::connect_to_server();
        super::qualify(&pic);
    }
}
