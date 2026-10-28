//! Explicit installed qualification of the supplied current-graph fixture.
mod ownership;
mod storage;

use clap::Parser;
use ic_platform::pocket_ic::{ErrorCode, PocketIcBuilder, RejectResponse};
use std::{env, fs, time::Duration};

///
/// Arguments
///
/// Select measurement-only execution before reading inputs or contacting a server.
///

#[derive(Debug, Parser)]
#[command(about = "Qualify the supplied Wasm against an explicitly prepared PocketIC server")]
struct Arguments {
    /// Measure valid IO only, skipping rejection and upgrade checks.
    #[arg(long)]
    measure_valid_io: bool,
}

/// Decoded fields of the supplied Memory fixture's exact 26-byte snapshot.
#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    generation: u64,
    markers: [u8; 2],
    physical_pages: u64,
    rows_pages: u64,
}

impl Snapshot {
    fn decode(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 26, "unexpected Memory snapshot shape");
        Self {
            generation: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            markers: bytes[8..10].try_into().unwrap(),
            physical_pages: u64::from_le_bytes(bytes[10..18].try_into().unwrap()),
            rows_pages: u64::from_le_bytes(bytes[18..].try_into().unwrap()),
        }
    }
}

fn io_request(operation: u8, offset: u64, count: u8) -> Vec<u8> {
    let mut bytes = vec![operation, count];
    bytes.extend_from_slice(&offset.to_le_bytes());
    bytes
}

fn io_instructions(reply: &[u8]) -> u64 {
    assert_eq!(reply.len(), 10, "unexpected IO reply shape");
    u64::from_le_bytes(reply[2..].try_into().unwrap())
}

fn assert_trap(error: RejectResponse) {
    assert!(
        matches!(
            error.error_code,
            ErrorCode::CanisterTrapped | ErrorCode::CanisterCalledTrap
        ),
        "expected an execution trap, got {error:?}",
    );
}

fn main() {
    let arguments = Arguments::parse();
    let wasm = fs::read(env::var("IC_MEMORY_QUALIFICATION_WASM").expect("fixture path")).unwrap();
    // The caller owns server startup/teardown. Never implicitly spawn or
    // download a binary as part of qualification.
    let url = env::var("IC_TESTKIT_POCKET_IC_URL").expect("caller-selected PocketIC server URL");
    let pic = PocketIcBuilder::new()
        .with_server_url(url.parse().unwrap())
        .with_application_subnet()
        .build();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm.clone(), Vec::new(), None);
    let snapshot = || {
        Snapshot::decode(
            &pic.query_call(canister, canister, "snapshot", Vec::new())
                .unwrap(),
        )
    };
    let initial = snapshot();
    assert_eq!(initial.generation, 1);
    assert_eq!(initial.markers, [0x42, 0x6D]);
    assert_eq!(initial.rows_pages, 1);

    if !arguments.measure_valid_io {
        // Check all three delegates, including empty spans beyond the extent.
        for operation in 0..3 {
            for (offset, count) in [(65_536, 1), (65_537, 0), (u64::MAX, 1), (u64::MAX - 1, 2)] {
                let args = io_request(operation, offset, count);
                assert_trap(pic.update_call(canister, canister, "io", args).unwrap_err());
                assert_eq!(
                    snapshot(),
                    initial,
                    "invalid IO must roll back the preceding valid write"
                );
            }
        }
        println!("installed invalid IO: 12 traps, neighboring marker and prior mutation preserved");

        for mode in [1, 2] {
            pic.advance_time(Duration::from_secs(300));
            assert_trap(
                pic.upgrade_canister(canister, wasm.clone(), vec![mode], None)
                    .unwrap_err(),
            );
            assert_eq!(
                snapshot(),
                initial,
                "failed upgrade must preserve original state"
            );
        }
        pic.advance_time(Duration::from_secs(300));
        pic.upgrade_canister(canister, wasm.clone(), vec![0], None)
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
            let args = io_request(operation, offset, count);
            let cycles_before = pic.cycle_balance(canister);
            let reply = pic.update_call(canister, canister, "io", args).unwrap();
            let cycles = cycles_before
                .checked_sub(pic.cycle_balance(canister))
                .expect("measured update must not increase the cycle balance");
            let instructions = io_instructions(&reply);
            println!("valid_io,{operation},{offset},{count},1000,{instructions}");
            println!("valid_io_cycles,{operation},{offset},{count},1000,{cycles}");
        }
    }
    let final_snapshot = snapshot();
    assert_eq!(final_snapshot.markers, [0x99, 0x6D]);
    assert_eq!(final_snapshot.physical_pages, initial.physical_pages);
    assert_eq!(final_snapshot.rows_pages, initial.rows_pages);
    println!("fixture_wasm_bytes,{}", wasm.len());
    if !arguments.measure_valid_io {
        storage::qualify(&pic);
        ownership::qualify(&pic);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_framing_preserves_bytes_and_refuses_short_or_trailing_replies() {
        assert_eq!(
            io_request(2, 0x0102_0304_0506_0708, 1),
            [2, 1, 8, 7, 6, 5, 4, 3, 2, 1]
        );
        assert_eq!(
            io_instructions(&[0x42, 0x6D, 8, 7, 6, 5, 4, 3, 2, 1]),
            0x0102_0304_0506_0708
        );
        for length in [0, 2, 9, 11] {
            assert!(std::panic::catch_unwind(|| io_instructions(&vec![0; length])).is_err());
        }
    }

    #[test]
    fn snapshot_decode_preserves_fields_and_refuses_truncated_or_trailing_bytes() {
        let bytes = [
            1, 0, 0, 0, 0, 0, 0, 0, 0x42, 0x6D, 9, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(
            Snapshot::decode(&bytes),
            Snapshot {
                generation: 1,
                markers: [0x42, 0x6D],
                physical_pages: 9,
                rows_pages: 1,
            }
        );
        for length in [0, 8, 25, 27] {
            assert!(std::panic::catch_unwind(|| Snapshot::decode(&vec![0; length])).is_err());
        }
    }

    #[test]
    fn measurement_arguments_reject_extra_tokens() {
        assert!(
            !Arguments::try_parse_from(["runner"])
                .unwrap()
                .measure_valid_io
        );
        assert!(
            Arguments::try_parse_from(["runner", "--measure-valid-io"])
                .unwrap()
                .measure_valid_io
        );
        for args in [
            vec!["runner", "unexpected"],
            vec!["runner", "--measure-valid-io", "unexpected"],
            vec!["runner", "--unknown"],
        ] {
            assert!(Arguments::try_parse_from(args).is_err());
        }
    }
}
