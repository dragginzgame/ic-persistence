//! Same-canister backing custody and retained component handles.
use ic_platform::pocket_ic::PocketIc;

pub(super) fn qualify(pic: &PocketIc) {
    let wasm =
        std::fs::read(std::env::var("IC_MEMORY_QUALIFICATION_WASM").expect("Memory fixture path"))
            .unwrap();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm.clone(), Vec::new(), None);
    let snapshot = || {
        super::Snapshot::decode(
            &pic.query_call(canister, canister, "snapshot", Vec::new())
                .unwrap(),
        )
    };
    let initial = snapshot();
    for mode in 0..3 {
        super::assert_trap(
            pic.update_call(canister, canister, "metadata_fault", vec![mode])
                .unwrap_err(),
        );
        assert_eq!(
            snapshot(),
            initial,
            "manager fault and preceding payload write must roll back"
        );
    }
    pic.update_call(canister, canister, "composition", Vec::new())
        .unwrap();
    assert_eq!(
        snapshot(),
        super::Snapshot {
            generation: initial.generation + 1,
            ..initial
        }
    );
    pic.advance_time(std::time::Duration::from_secs(300));
    pic.upgrade_canister(canister, wasm, vec![0], None).unwrap();
    assert_eq!(
        snapshot(),
        super::Snapshot {
            generation: initial.generation + 2,
            ..initial
        }
    );
    pic.update_call(canister, canister, "composition", Vec::new())
        .unwrap();
    assert_eq!(
        snapshot(),
        super::Snapshot {
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
    use ic_platform::pocket_ic::PocketIcBuilder;
    #[test]
    #[ignore = "requires an explicitly prepared server URL and Memory fixture Wasm"]
    fn installed_backing_custody() {
        let url = std::env::var("IC_TESTKIT_POCKET_IC_URL").expect("caller-selected server URL");
        let pic = PocketIcBuilder::new()
            .with_server_url(url.parse().unwrap())
            .with_application_subnet()
            .build();
        qualify(&pic);
    }
}
