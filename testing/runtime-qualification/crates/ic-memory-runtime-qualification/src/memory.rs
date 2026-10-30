//! Installed Memory scenarios and actual valid-IO measurements.
pub(super) mod protocol {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/ic-memory/examples/support/io_protocol.rs"
    ));
}
use ic_platform::{pocket_ic::PocketIc, principal::ExternalPrincipal};
use ic_wire::native;
use protocol::Snapshot;
use std::{env, fs, time::Duration};

pub(super) fn frame(id: u64, value: &impl serde::Serialize) -> Vec<u8> {
    native::encode_request(id, value, protocol::REQUEST).unwrap()
}
pub(super) fn read(pic: &PocketIc, canister: ExternalPrincipal) -> Snapshot {
    let bytes = pic
        .query_call(
            canister,
            canister,
            "snapshot",
            frame(protocol::SNAPSHOT_ID, &()),
        )
        .unwrap();
    native::decode_response(protocol::SNAPSHOT_ID, &bytes, protocol::RESPONSE).unwrap()
}

pub(super) fn qualify(pic: &PocketIc, measure_valid_io: bool) {
    let wasm = fs::read(env::var("IC_MEMORY_QUALIFICATION_WASM").expect("fixture path")).unwrap();
    let canister = super::install_fixture(pic, &wasm, frame(protocol::INIT_ID, &()));
    let snapshot = || read(pic, canister);
    let initial = snapshot();
    assert_eq!(initial.generation, 1);
    assert_eq!(initial.markers, [0x42, 0x6D]);
    assert_eq!(initial.rows_pages, 1);

    if !measure_valid_io {
        let valid = frame(
            protocol::IO_ID,
            &protocol::IoRequest {
                operation: 0,
                count: 1,
                offset: 7,
            },
        );
        let mut trailing = valid.clone();
        trailing.push(0);
        for bytes in [
            Vec::new(),
            valid[..valid.len() - 1].to_vec(),
            trailing,
            frame(protocol::SNAPSHOT_ID, &()),
            vec![0; protocol::REQUEST.bytes + 1],
        ] {
            super::assert_trap(
                pic.update_call(canister, canister, "io", bytes)
                    .unwrap_err(),
            );
            assert_eq!(
                snapshot(),
                initial,
                "refused Wire frame must preserve state"
            );
        }
        println!(
            "installed Memory Wire: five malformed/foreign/oversized frames refuse without retained mutation"
        );
        // Check all three delegates, including empty spans beyond the extent.
        for operation in 0..3 {
            for (offset, count) in [(65_536, 1), (65_537, 0), (u64::MAX, 1), (u64::MAX - 1, 2)] {
                let args = frame(
                    protocol::IO_ID,
                    &protocol::IoRequest {
                        operation,
                        count,
                        offset,
                    },
                );
                super::assert_trap(pic.update_call(canister, canister, "io", args).unwrap_err());
                assert_eq!(
                    snapshot(),
                    initial,
                    "invalid IO must roll back the preceding valid write"
                );
            }
        }
        println!("installed invalid IO: 12 traps, neighboring marker and prior mutation preserved");

        for mode in [1_u8, 2] {
            pic.advance_time(Duration::from_secs(300));
            super::assert_trap(
                pic.upgrade_canister(
                    canister,
                    wasm.clone(),
                    frame(protocol::UPGRADE_ID, &mode),
                    None,
                )
                .unwrap_err(),
            );
            assert_eq!(
                snapshot(),
                initial,
                "failed upgrade must preserve original state"
            );
        }
        pic.advance_time(Duration::from_secs(300));
        pic.upgrade_canister(
            canister,
            wasm.clone(),
            frame(protocol::UPGRADE_ID, &0_u8),
            None,
        )
        .unwrap();
        let upgraded = snapshot();
        assert_eq!(
            upgraded,
            Snapshot {
                generation: 2,
                ..initial
            }
        );
        println!(
            "installed upgrade: post-persistence trap and geometry conflict roll back; clean retry advances exactly one generation"
        );
    }

    for operation in 0..3 {
        for (offset, count) in [(7_u64, 1), (65_536, 0), (65_535, 1)] {
            let args = frame(
                protocol::IO_ID,
                &protocol::IoRequest {
                    operation,
                    count,
                    offset,
                },
            );
            let cycles_before = pic.cycle_balance(canister);
            let reply = pic.update_call(canister, canister, "io", args).unwrap();
            let cycles = cycles_before
                .checked_sub(pic.cycle_balance(canister))
                .expect("measured update must not increase the cycle balance");
            let reply: protocol::IoReply =
                native::decode_response(protocol::IO_ID, &reply, protocol::RESPONSE).unwrap();
            let instructions = reply.instructions;
            println!("valid_io,{operation},{offset},{count},1000,{instructions}");
            println!("valid_io_cycles,{operation},{offset},{count},1000,{cycles}");
        }
    }
    let final_snapshot = snapshot();
    assert_eq!(final_snapshot.markers, [0x99, 0x6D]);
    assert_eq!(final_snapshot.physical_pages, initial.physical_pages);
    assert_eq!(final_snapshot.rows_pages, initial.rows_pages);
    println!("fixture_wasm_bytes,{}", wasm.len());
}
