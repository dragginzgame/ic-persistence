//! Installed Storage adapters under the same caller-owned server as Memory.
mod protocol {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/ic-storage/examples/support/qualification_protocol.rs"
    ));
}
use ic_platform::{
    pocket_ic::{PocketIc, RejectResponse},
    principal::ExternalPrincipal,
};
use ic_wire::native;
fn frame(id: u64, value: &impl serde::Serialize) -> Vec<u8> {
    native::encode_request(id, value, protocol::REQUEST).unwrap()
}
use std::{env, fs, time::Duration};

fn update(
    pic: &PocketIc,
    canister: ExternalPrincipal,
    method: &str,
    id: u64,
    mode: u8,
) -> Result<(), RejectResponse> {
    pic.update_call(canister, canister, method, frame(id, &mode))
        .map(|bytes| {
            native::decode_response::<()>(id, &bytes, protocol::RESPONSE).unwrap();
        })
}

pub(super) fn qualify(pic: &PocketIc) {
    let wasm =
        fs::read(env::var("IC_STORAGE_QUALIFICATION_WASM").expect("Storage fixture path")).unwrap();
    let canister = super::install_fixture(pic, &wasm, frame(protocol::INIT_ID, &()));
    let snapshot = || {
        let bytes = pic
            .query_call(
                canister,
                canister,
                "snapshot",
                frame(protocol::SNAPSHOT_ID, &()),
            )
            .unwrap();
        native::decode_response::<protocol::Snapshot>(
            protocol::SNAPSHOT_ID,
            &bytes,
            protocol::RESPONSE,
        )
        .unwrap()
    };
    let expected = |generation, values| protocol::Snapshot { generation, values };
    let initial = snapshot();
    assert_eq!(initial, expected(1, [10, 20, 30]));
    let mut trailing = frame(protocol::SNAPSHOT_ID, &());
    trailing.push(0);
    for bytes in [
        Vec::new(),
        trailing,
        frame(protocol::MUTATE_ID, &()),
        vec![0; protocol::REQUEST.bytes + 1],
    ] {
        super::assert_trap(
            pic.query_call(canister, canister, "snapshot", bytes)
                .unwrap_err(),
        );
        assert_eq!(
            snapshot(),
            initial,
            "refused Wire snapshot request must preserve state"
        );
    }
    println!("installed Storage Wire: four malformed/foreign/oversized snapshot frames refuse");
    for mode in 0..4_u8 {
        update(
            pic,
            canister,
            "cell_load_failure",
            protocol::CELL_FAILURE_ID,
            mode,
        )
        .unwrap();
        assert_eq!(
            snapshot(),
            initial,
            "rejected cell load preserves live collections"
        );
    }
    super::assert_trap(
        update(
            pic,
            canister,
            "cell_load_failure",
            protocol::CELL_FAILURE_ID,
            4_u8,
        )
        .unwrap_err(),
    );
    assert_eq!(
        snapshot(),
        initial,
        "cell decoder trap preserves live collections"
    );
    // The failed call's lease is rolled back with the Wasm heap and can reopen.
    update(
        pic,
        canister,
        "cell_load_failure",
        protocol::CELL_FAILURE_ID,
        3_u8,
    )
    .unwrap();
    super::assert_trap(update(pic, canister, "mutate", protocol::MUTATE_ID, 1_u8).unwrap_err());
    assert_eq!(
        snapshot(),
        initial,
        "trapped update rolls back all collection writes"
    );
    update(pic, canister, "mutate", protocol::MUTATE_ID, 0_u8).unwrap();
    let committed = snapshot();
    assert_eq!(committed, expected(1, [100, 101, 102]));
    pic.advance_time(Duration::from_secs(300));
    super::assert_trap(
        pic.upgrade_canister(
            canister,
            wasm.clone(),
            frame(protocol::UPGRADE_ID, &1_u8),
            None,
        )
        .unwrap_err(),
    );
    assert_eq!(
        snapshot(),
        committed,
        "failed upgrade preserves collections and generation"
    );
    pic.advance_time(Duration::from_secs(300));
    pic.upgrade_canister(canister, wasm, frame(protocol::UPGRADE_ID, &0_u8), None)
        .unwrap();
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
