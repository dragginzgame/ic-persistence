//! Ledger codec contracts.

use super::*;

#[test]
fn unsupported_discriminator_precedes_logical_decode_and_preserves_storage() {
    let mut committed = CommittedGenerationBytes::new(1, vec![0xff]);
    committed.commit_marker = u64::MAX;
    let store = LedgerCommitStore {
        current: Some(committed),
    };
    let before = store.clone();
    let expected =
        LedgerCommitError::Recovery(CommitRecoveryError::UnsupportedFormat { marker: u64::MAX });
    assert_eq!(store.recover(), Err(expected.clone()));
    assert_eq!(store.recover_or_genesis(), Err(expected));
    assert_eq!(store, before);
}

#[test]
fn ledger_wire_round_trips_allocation_ledger() {
    let ledger = committed_ledger(2);

    let encoded = codec::encode_ledger(&ledger).expect("encode ledger");
    let decoded = decode_ledger(&encoded, 2).expect("decode ledger");

    assert_eq!(decoded, ledger);
}

#[test]
fn ledger_wire_rejects_trailing_bytes() {
    let mut encoded = codec::encode_ledger(&committed_ledger(2)).expect("encode ledger");
    encoded.push(0);

    let err = decode_ledger(&encoded, 2).expect_err("trailing bytes must fail closed");

    assert_eq!(err, ic_wire::Error::Trailing.to_string());
}

#[test]
fn wire_ledger_codec_rejects_truncated_and_excess_records() {
    for bytes in [vec![], vec![1], vec![128, 2], vec![255; 10]] {
        assert!(decode_ledger(&bytes, 1).is_err());
    }
}

#[test]
fn wire_ledger_codec_rejects_extra_record_data() {
    #[derive(serde::Serialize)]
    struct ExtraRecord<'a> {
        stable_key: &'a str,
        slot: u8,
        extra: bool,
    }
    let bytes = crate::test_wire::to_vec(&vec![ExtraRecord {
        stable_key: "app.users.v1",
        slot: 100,
        extra: true,
    }])
    .unwrap();
    assert!(decode_ledger(&bytes, 1).is_err());
}

#[test]
fn ledger_commit_store_rejects_extra_top_level_data() {
    let mut bytes = crate::test_wire::to_vec(&LedgerCommitStore::default()).unwrap();
    bytes.push(1);
    assert_eq!(
        crate::test_wire::from_slice::<LedgerCommitStore>(&bytes),
        Err(ic_wire::Error::Trailing)
    );
}

#[test]
fn current_empty_genesis_payload_fixture_recovers() {
    let ledger = ledger_from_payload_fixture(
        include_str!("../../../fixtures/current/empty_genesis_ledger.wire.hex"),
        0,
    );

    assert_eq!(ledger.current_generation, 0);
    assert_eq!(ledger.records(), []);
}

#[test]
fn current_single_payload_fixture_recovers() {
    let ledger = ledger_from_payload_fixture(
        include_str!("../../../fixtures/current/single_allocation_ledger.wire.hex"),
        1,
    );
    let record = record(&ledger, "app.users.v1");

    assert_eq!(ledger.current_generation, 1);
    assert_eq!(record.slot().id(), 100);
}

#[test]
fn current_memory_manager_slot_fixture_decodes() {
    let bytes = hex_fixture(include_str!(
        "../../../fixtures/current/memory_manager_slot.wire.hex"
    ));
    let slot: MemoryManagerSlot = crate::test_wire::from_slice(&bytes).expect("slot fixture");

    assert_eq!(slot.id(), 100);
    assert_eq!(
        bytes,
        crate::test_wire::to_vec(&slot).expect("re-encoded slot")
    );
}

#[test]
fn current_ledger_commit_store_single_fixture_recovers() {
    let store = store_from_fixture(include_str!(
        "../../../fixtures/current/ledger_commit_store_single.wire.hex"
    ));

    let recovered = store.recover().expect("fixture store recovers");
    assert_eq!(recovered.current_generation(), 1);
    assert_eq!(record(recovered.ledger(), "app.users.v1").slot().id(), 100);
}

#[test]
fn current_snapshot_fixture_recovers() {
    let store = store_from_fixture(include_str!(
        "../../../fixtures/current/snapshot_store_valid.wire.hex"
    ));

    assert_eq!(
        store.diagnostic(),
        crate::CommitSnapshotDiagnostic::Valid { generation: 2 }
    );
    let recovered = store.recover().expect("fixture store recovers");
    assert_eq!(recovered.current_generation(), 2);
}

#[test]
fn current_corrupt_snapshot_fixture_fails_closed() {
    let store = store_from_fixture(include_str!(
        "../../../fixtures/current/snapshot_store_corrupt.wire.hex"
    ));

    let diagnostic = store.diagnostic();
    assert_eq!(
        diagnostic,
        crate::CommitSnapshotDiagnostic::Invalid { generation: 2 }
    );
    let err = store
        .recover()
        .expect_err("corrupt fixture must fail closed");
    assert!(matches!(
        err,
        LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 2 })
    ));
}

#[test]
fn oversized_logical_text_remains_opaque_to_record_decode_and_fails_closed() {
    for len in [257, 8192] {
        let payload = crate::test_wire::to_vec(&vec![("x".repeat(len), 100_u8)]).unwrap();
        let mut store = LedgerCommitStore::default();
        store.commit_payload_at_generation(1, payload).unwrap();
        let record = store;
        let bytes = crate::stable_cell::EncodedLedgerRecord::new(&record)
            .unwrap()
            .into_bytes();
        let decoded = crate::decode_stable_cell_commit_store(&bytes).unwrap();
        assert_eq!(decoded, record);
        let before = decoded.clone();
        assert!(matches!(
            decoded.recover(),
            Err(LedgerCommitError::Codec(_))
        ));
        assert!(matches!(
            decoded.recover_or_genesis(),
            Err(LedgerCommitError::Codec(_))
        ));
        assert_eq!(decoded, before);
    }
}

// Regeneration is an explicit source-edit operation, never a normal test effect.
#[test]
#[ignore = "explicit current Wire fixture regeneration"]
fn regenerate_current_wire_fixtures() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/current");
    let write = |name: &str, bytes: Vec<u8>| {
        use std::fmt::Write as _;
        let mut hex = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            write!(&mut hex, "{byte:02x}").unwrap();
        }
        std::fs::write(directory.join(name), format!("{hex}\n")).unwrap();
    };
    write(
        "empty_genesis_ledger.wire.hex",
        encoded_payload(&committed_ledger(0)),
    );
    write(
        "single_allocation_ledger.wire.hex",
        encoded_payload(&committed_fixture_ledger()),
    );
    write(
        "memory_manager_slot.wire.hex",
        crate::test_wire::to_vec(&MemoryManagerSlot::new(100).unwrap()).unwrap(),
    );
    let mut store = LedgerCommitStore::default();
    store.commit(&committed_fixture_ledger()).unwrap();
    write(
        "ledger_commit_store_single.wire.hex",
        crate::stable_cell::EncodedLedgerRecord::new(&store)
            .unwrap()
            .into_bytes(),
    );
    let mut second = committed_fixture_ledger();
    second.current_generation = 2;
    store.commit(&second).unwrap();
    write(
        "snapshot_store_valid.wire.hex",
        crate::stable_cell::EncodedLedgerRecord::new(&store)
            .unwrap()
            .into_bytes(),
    );
    store.current.as_mut().unwrap().checksum ^= 1;
    write(
        "snapshot_store_corrupt.wire.hex",
        crate::stable_cell::EncodedLedgerRecord::new(&store)
            .unwrap()
            .into_bytes(),
    );
}
