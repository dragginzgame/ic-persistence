//! Explicit installed qualification of the supplied current-graph fixture.
mod logging;
mod memory;
mod ownership;
mod storage;

use clap::Parser;
use ic_platform::{
    pocket_ic::{ErrorCode, PocketIc, PocketIcBuilder, RejectResponse},
    principal::ExternalPrincipal,
};
use std::env;

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
    /// Qualify only the supplied logging fixture, without Memory/Storage suites.
    #[arg(long, conflicts_with = "measure_valid_io")]
    logging_only: bool,
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

fn connect_to_server() -> PocketIc {
    // The caller owns server startup/teardown. Never implicitly spawn or
    // download a binary as part of qualification.
    let url = env::var("IC_TESTKIT_POCKET_IC_URL").expect("caller-selected PocketIC server URL");
    PocketIcBuilder::new()
        .with_server_url(url.parse().unwrap())
        .with_application_subnet()
        .build()
}

fn install_fixture(pic: &PocketIc, wasm: &[u8], init: Vec<u8>) -> ExternalPrincipal {
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm.to_vec(), init, None);
    canister
}

fn main() {
    let arguments = Arguments::parse();
    let pic = connect_to_server();
    if arguments.logging_only {
        logging::qualify(&pic);
    } else {
        memory::qualify(&pic, arguments.measure_valid_io);
        if !arguments.measure_valid_io {
            storage::qualify(&pic);
            ownership::qualify(&pic);
            logging::qualify(&pic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_wire_frames_admit_extremes_and_refuse_identity_trailing_and_budget_errors() {
        use ic_wire::native;
        use memory::protocol;
        let request = protocol::IoRequest {
            operation: 2,
            count: 1,
            offset: u64::MAX,
        };
        let bytes = memory::frame(protocol::IO_ID, &request);
        assert_eq!(
            native::decode_request::<protocol::IoRequest>(
                protocol::IO_ID,
                &bytes,
                protocol::REQUEST
            )
            .unwrap(),
            request
        );
        assert!(
            native::decode_request::<protocol::IoRequest>(
                protocol::SNAPSHOT_ID,
                &bytes,
                protocol::REQUEST
            )
            .is_err()
        );
        for end in 0..bytes.len() {
            assert!(
                native::decode_request::<protocol::IoRequest>(
                    protocol::IO_ID,
                    &bytes[..end],
                    protocol::REQUEST
                )
                .is_err()
            );
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(
            native::decode_request::<protocol::IoRequest>(
                protocol::IO_ID,
                &trailing,
                protocol::REQUEST
            )
            .is_err()
        );
        let oversized = vec![0; protocol::REQUEST.bytes + 1];
        assert!(
            native::decode_request::<protocol::IoRequest>(
                protocol::IO_ID,
                &oversized,
                protocol::REQUEST
            )
            .is_err()
        );
        let snapshot = protocol::Snapshot {
            generation: u64::MAX,
            markers: [0x42, 0x6d],
            physical_pages: 9,
            rows_pages: 1,
        };
        let response =
            native::encode_response(protocol::SNAPSHOT_ID, &snapshot, protocol::RESPONSE).unwrap();
        assert_eq!(
            native::decode_response::<protocol::Snapshot>(
                protocol::SNAPSHOT_ID,
                &response,
                protocol::RESPONSE
            )
            .unwrap(),
            snapshot
        );
        let empty = memory::frame(protocol::INIT_ID, &());
        assert!(native::decode_request::<()>(protocol::INIT_ID, &empty, protocol::REQUEST).is_ok());
    }

    #[test]
    fn measurement_arguments_reject_extra_tokens() {
        assert!(
            Arguments::try_parse_from(["runner", "--logging-only"])
                .unwrap()
                .logging_only
        );
        assert!(
            Arguments::try_parse_from(["runner", "--logging-only", "--measure-valid-io"]).is_err()
        );
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
