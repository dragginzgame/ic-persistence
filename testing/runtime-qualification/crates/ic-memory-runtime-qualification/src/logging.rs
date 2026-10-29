//! Focused installed Logging checkpoints using the fixture's shared Wire contract.
mod protocol {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/ic-logging-stable/examples/logging_qualification/protocol.rs"
    ));
}

use ic_platform::{pocket_ic::PocketIc, principal::ExternalPrincipal};
use ic_wire::native;
use protocol::{Append, Outcome, Reply, Snapshot, Update, Upgrade};
use serde::Serialize;
use std::{env, fs, time::Duration};

fn frame(id: u64, value: &impl Serialize) -> Vec<u8> {
    native::encode_request(id, value, protocol::REQUEST).unwrap()
}
fn read(pic: &PocketIc, canister: ExternalPrincipal) -> Snapshot {
    query(pic, canister, None).snapshot
}
fn query(pic: &PocketIc, canister: ExternalPrincipal, input: Option<Append>) -> Reply {
    let bytes = pic
        .query_call(
            canister,
            canister,
            "logging_query",
            frame(protocol::QUERY_ID, &input),
        )
        .unwrap();
    native::decode_response(protocol::QUERY_ID, &bytes, protocol::RESPONSE).unwrap()
}
fn update(pic: &PocketIc, canister: ExternalPrincipal, input: &Update) -> Reply {
    let bytes = pic
        .update_call(
            canister,
            canister,
            "logging_update",
            frame(protocol::UPDATE_ID, input),
        )
        .unwrap();
    native::decode_response(protocol::UPDATE_ID, &bytes, protocol::RESPONSE).unwrap()
}
fn append(time: u64, trap: bool) -> Update {
    Update::Append(Append {
        created_at_secs: time,
        message: "retained event".into(),
        trap_after_save: trap,
    })
}

pub(super) fn qualify(pic: &PocketIc) {
    let wasm =
        fs::read(env::var("IC_LOGGING_QUALIFICATION_WASM").expect("logging fixture path")).unwrap();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm.clone(), frame(protocol::INIT_ID, &()), None);
    let initial = read(pic, canister);
    assert!(initial.rows.is_empty());
    assert_eq!(initial.next_sequence, Some(0));
    assert_eq!(initial.last_created_at_secs, None);

    let ephemeral = query(
        pic,
        canister,
        Some(Append {
            created_at_secs: 10,
            message: "é🦀".repeat(20),
            trap_after_save: false,
        }),
    );
    assert_eq!(ephemeral.outcome, Outcome::Appended(0));
    assert!(ephemeral.snapshot.rows[0].truncated);
    assert_eq!(
        read(pic, canister),
        initial,
        "query checkpoint must not persist"
    );

    let first = update(pic, canister, &append(20, false));
    assert_eq!(first.outcome, Outcome::Appended(0));
    let retained = first.snapshot;
    super::assert_trap(
        pic.update_call(
            canister,
            canister,
            "logging_update",
            frame(protocol::UPDATE_ID, &append(21, true)),
        )
        .unwrap_err(),
    );
    assert_eq!(
        read(pic, canister),
        retained,
        "trapped update must roll back heap and checkpoint"
    );

    pic.advance_time(Duration::from_secs(300));
    super::assert_trap(
        pic.upgrade_canister(
            canister,
            wasm.clone(),
            frame(protocol::UPGRADE_ID, &Upgrade::TrapAfterCheckpoint),
            None,
        )
        .unwrap_err(),
    );
    assert_eq!(
        read(pic, canister),
        retained,
        "trapped upgrade must roll back its checkpoint"
    );
    pic.advance_time(Duration::from_secs(300));
    pic.upgrade_canister(
        canister,
        wasm.clone(),
        frame(protocol::UPGRADE_ID, &Upgrade::Restore),
        None,
    )
    .unwrap();
    assert_eq!(read(pic, canister), retained);
    assert_eq!(
        update(pic, canister, &append(19, false)).outcome,
        Outcome::TimestampRefused
    );
    assert_eq!(read(pic, canister), retained);
    assert_eq!(
        update(pic, canister, &append(20, false)).outcome,
        Outcome::Appended(1)
    );
    let pruned = update(
        pic,
        canister,
        &Update::Prune {
            now_secs: 21,
            max_age_secs: 0,
            max_remove: 4,
        },
    );
    assert_eq!(pruned.outcome, Outcome::Pruned(2));
    assert!(pruned.snapshot.rows.is_empty());
    assert_eq!(pruned.snapshot.next_sequence, Some(2));
    assert_eq!(pruned.snapshot.last_created_at_secs, Some(20));
    pic.advance_time(Duration::from_secs(300));
    pic.upgrade_canister(
        canister,
        wasm,
        frame(protocol::UPGRADE_ID, &Upgrade::Restore),
        None,
    )
    .unwrap();
    assert_eq!(read(pic, canister), pruned.snapshot);
    assert_eq!(
        update(pic, canister, &append(19, false)).outcome,
        Outcome::TimestampRefused
    );
    let resumed = update(pic, canister, &append(20, false));
    assert_eq!(resumed.outcome, Outcome::Appended(2));
    assert_eq!(resumed.snapshot.next_sequence, Some(3));
    let wrong_identity = frame(protocol::UPDATE_ID, &None::<Append>);
    assert!(
        pic.query_call(canister, canister, "logging_query", wrong_identity)
            .is_err()
    );
    assert_eq!(read(pic, canister), resumed.snapshot);
    println!(
        "installed logging: query non-persistence, update/upgrade rollback, reopen and empty-history watermarks pass"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_wire_admits_maximum_fixture_rows_and_binds_lifecycle_identity() {
        let value = Reply {
            outcome: Outcome::Appended(u64::MAX),
            snapshot: Snapshot {
                rows: vec![
                    protocol::Row {
                        sequence: u64::MAX,
                        created_at_secs: u64::MAX,
                        message: "x".repeat(64),
                        truncated: true
                    };
                    4
                ],
                next_sequence: None,
                last_created_at_secs: Some(u64::MAX),
            },
        };
        let bytes =
            native::encode_response(protocol::QUERY_ID, &value, protocol::RESPONSE).unwrap();
        assert_eq!(
            native::decode_response::<Reply>(protocol::QUERY_ID, &bytes, protocol::RESPONSE)
                .unwrap(),
            value
        );
        let bytes = frame(protocol::UPGRADE_ID, &Upgrade::Restore);
        assert!(
            native::decode_request::<Upgrade>(protocol::INIT_ID, &bytes, protocol::REQUEST)
                .is_err()
        );
        assert!(
            native::decode_request::<Upgrade>(protocol::UPGRADE_ID, &bytes, protocol::REQUEST)
                .is_ok()
        );
    }
}
