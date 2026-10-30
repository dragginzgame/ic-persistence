//! Same-canister backing custody and retained component handles.
use super::memory::{frame, protocol, read};
use ic_platform::pocket_ic::PocketIc;

pub(super) fn qualify(pic: &PocketIc) {
    let wasm =
        std::fs::read(std::env::var("IC_MEMORY_QUALIFICATION_WASM").expect("Memory fixture path"))
            .unwrap();
    let canister = super::install_fixture(pic, &wasm, frame(protocol::INIT_ID, &()));
    let snapshot = || read(pic, canister);
    let initial = snapshot();
    for mode in 0..6_u8 {
        super::assert_trap(
            pic.update_call(
                canister,
                canister,
                "metadata_fault",
                frame(protocol::METADATA_FAULT_ID, &mode),
            )
            .unwrap_err(),
        );
        assert_eq!(
            snapshot(),
            initial,
            "manager fault and preceding payload write must roll back"
        );
    }
    println!(
        "installed manager checkpoints: six open/growth faults trap, poison old handles and roll back metadata/payload writes"
    );
    let reply = pic
        .update_call(
            canister,
            canister,
            "composition",
            frame(protocol::COMPOSITION_ID, &()),
        )
        .unwrap();
    ic_wire::native::decode_response::<()>(protocol::COMPOSITION_ID, &reply, protocol::RESPONSE)
        .unwrap();
    assert_eq!(
        snapshot(),
        protocol::Snapshot {
            generation: initial.generation + 1,
            ..initial
        }
    );
    pic.advance_time(std::time::Duration::from_secs(300));
    pic.upgrade_canister(canister, wasm, frame(protocol::UPGRADE_ID, &0_u8), None)
        .unwrap();
    assert_eq!(
        snapshot(),
        protocol::Snapshot {
            generation: initial.generation + 2,
            ..initial
        }
    );
    let reply = pic
        .update_call(
            canister,
            canister,
            "composition",
            frame(protocol::COMPOSITION_ID, &()),
        )
        .unwrap();
    ic_wire::native::decode_response::<()>(protocol::COMPOSITION_ID, &reply, protocol::RESPONSE)
        .unwrap();
    assert_eq!(
        snapshot(),
        protocol::Snapshot {
            generation: initial.generation + 3,
            ..initial
        }
    );
    println!(
        "installed backing custody: composed handles refuse duplicate owners through runtime drop; final release and upgrade reopen preserve payloads/pages"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an explicitly prepared server URL and Memory fixture Wasm"]
    fn installed_backing_custody() {
        let pic = crate::connect_to_server();
        qualify(&pic);
    }
}
