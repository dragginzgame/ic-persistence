//! Explicit installed qualification of the supplied current-graph fixture.
mod logging;
mod memory;
mod ownership;
mod storage;

use clap::Parser;
use ic_platform::{
    pocket_ic::{ClientError, ErrorCode, PocketIc, PocketIcBuilder},
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

fn assert_trap(error: ClientError) {
    let ClientError::Canister(error) = error else {
        panic!("expected a canister trap, got client failure: {error:?}");
    };
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
        .expect("connect to prepared PocketIC server")
}

fn install_fixture(pic: &PocketIc, wasm: &[u8], init: Vec<u8>) -> ExternalPrincipal {
    let canister = pic.create_canister().expect("create fixture canister");
    pic.add_cycles(canister, 100_000_000_000_000)
        .expect("fund fixture canister");
    pic.install_canister(canister, wasm.to_vec(), init, None)
        .expect("install fixture canister");
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
mod tests;
