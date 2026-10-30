//! Stable Cell envelope, Wire record and publication contracts.

use super::*;
use crate::test_wire::hex_fixture;
use ic_platform::stable_structures::{Cell, VectorMemory};

#[test]
fn publication_refuses_oversized_payload_before_cell_mutation() {
    let record = LedgerCommitStore {
        current: Some(crate::physical::CommittedGenerationBytes::new(
            1,
            vec![0; crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1],
        )),
    };
    assert!(EncodedLedgerRecord::new(&record).is_err());
}

#[test]
fn stable_cell_ledger_record_round_trips_through_cell() {
    let memory = VectorMemory::default();
    let record = LedgerCommitStore::default();
    drop(Cell::new(
        memory.clone(),
        EncodedLedgerRecord::new(&record).unwrap(),
    ));
    let payload = decode_stable_cell_payload(&memory).expect("decode stable cell payload");
    let decoded = decode_stable_cell_commit_store(&payload).unwrap();
    assert_eq!(decoded, record);
    assert_eq!(
        crate::decode_stable_cell_commit_store_from_memory(&memory).unwrap(),
        record
    );
}

#[test]
fn current_stable_cell_record_fixture_recovers() {
    let bytes = hex_fixture(include_str!(
        "../../fixtures/current/ledger_commit_store_single.wire.hex"
    ));
    let record = decode_stable_cell_commit_store(&bytes).expect("stable-cell fixture");

    assert_eq!(
        EncodedLedgerRecord::new(&record).unwrap().len(),
        bytes.len()
    );
    let encoded = EncodedLedgerRecord::new(&record).unwrap();
    assert!(matches!(encoded.to_bytes(), Cow::Borrowed(_)));
    let memory = VectorMemory::default();
    drop(Cell::new(memory.clone(), encoded));
    assert_eq!(decode_stable_cell_payload(&memory).unwrap(), bytes);
    assert_eq!(
        EncodedLedgerRecord::new(&record).unwrap().into_bytes(),
        bytes
    );
    assert_eq!(
        bytes,
        EncodedLedgerRecord::new(&record).unwrap().into_bytes()
    );
    assert_eq!(
        record
            .recover()
            .expect("fixture store recovers")
            .current_generation(),
        1
    );
}

#[test]
fn malformed_payload_bytes_fail_closed_without_mutating_memory() {
    use crate::encoding::RECORD_MAGIC;
    let original = hex_fixture(include_str!(
        "../../fixtures/current/ledger_commit_store_single.wire.hex"
    ));
    let mut cases = vec![original[..original.len() - 1].to_vec()];
    // Valid prefix and present snapshot followed by hostile/truncated integers.
    for tail in [vec![255; 32], vec![1], vec![2]] {
        let mut bytes = RECORD_MAGIC.to_vec();
        bytes.extend_from_slice(&tail);
        cases.push(bytes);
    }
    for bytes in cases {
        let memory = VectorMemory::default();
        memory.grow(1);
        memory.write(0, STABLE_CELL_MAGIC);
        memory.write(3, &[STABLE_CELL_LAYOUT_VERSION]);
        memory.write(4, &u32::try_from(bytes.len()).unwrap().to_le_bytes());
        memory.write(STABLE_CELL_VALUE_OFFSET, &bytes);
        let before = memory.borrow().clone();
        for _ in 0..2 {
            assert!(matches!(
                decode_stable_cell_commit_store_from_memory(&memory),
                Err(StableCellLedgerError::Record(_))
            ));
        }
        assert_eq!(*memory.borrow(), before);
    }
}

#[test]
fn stable_cell_ledger_record_rejects_trailing_bytes() {
    let mut bytes = EncodedLedgerRecord::new(&LedgerCommitStore::default())
        .unwrap()
        .into_bytes();
    bytes.push(0);

    let err = decode_stable_cell_commit_store(&bytes).expect_err("trailing bytes must fail closed");

    assert_eq!(err, ic_wire::Error::Trailing);
}

#[test]
fn stable_cell_ledger_record_rejects_extra_top_level_data() {
    let mut bytes = EncodedLedgerRecord::new(&LedgerCommitStore::default())
        .unwrap()
        .into_bytes();
    bytes.push(1);
    assert_eq!(
        decode_stable_cell_commit_store(&bytes),
        Err(ic_wire::Error::Trailing)
    );
}

#[test]
fn stable_cell_ledger_record_requires_an_explicit_current_snapshot() {
    assert!(decode_stable_cell_commit_store(crate::encoding::RECORD_MAGIC).is_err());
}

#[test]
fn stable_cell_payload_rejects_non_cell_memory() {
    let memory = VectorMemory::default();
    memory.grow(1);
    memory.write(0, b"BAD");

    assert_eq!(
        decode_stable_cell_payload(&memory),
        Err(StableCellPayloadError::NotStableCell)
    );
}

#[test]
fn stable_cell_payload_rejects_empty_memory_without_panic() {
    let memory = VectorMemory::default();

    assert_eq!(
        decode_stable_cell_payload(&memory),
        Err(StableCellPayloadError::NotStableCell)
    );
    assert_eq!(
        crate::decode_stable_cell_commit_store_from_memory(&memory).unwrap(),
        LedgerCommitStore::default()
    );
    assert_eq!(
        memory.size(),
        0,
        "reading empty memory must not initialize it"
    );
}

#[test]
fn stable_cell_ledger_memory_reader_classifies_bad_record_without_panic() {
    let memory = VectorMemory::default();
    memory.grow(1);
    memory.write(0, STABLE_CELL_MAGIC);
    memory.write(3, &[STABLE_CELL_LAYOUT_VERSION]);
    memory.write(4, &1_u32.to_le_bytes());
    memory.write(STABLE_CELL_VALUE_OFFSET, &[0xff]);

    let err = decode_stable_cell_commit_store_from_memory(&memory)
        .expect_err("bad record must be classified");

    assert!(matches!(err, StableCellLedgerError::Record(_)));
}
#[test]
fn oversized_cell_length_is_rejected_before_payload_read() {
    struct HeaderOnly {
        value_len: u32,
        pages: u64,
    }
    impl Memory for HeaderOnly {
        fn size(&self) -> u64 {
            self.pages
        }
        fn grow(&self, _: u64) -> i64 {
            panic!("must not grow")
        }
        fn write(&self, _: u64, _: &[u8]) {
            panic!("must not write")
        }
        fn read(&self, offset: u64, bytes: &mut [u8]) {
            assert_eq!(offset, 0);
            assert_eq!(bytes.len(), STABLE_CELL_HEADER_SIZE);
            bytes[..4].copy_from_slice(b"SCL\x01");
            bytes[4..8].copy_from_slice(&self.value_len.to_le_bytes());
        }
    }
    for value_len in [
        u32::try_from(crate::constants::MAX_LEDGER_RECORD_BYTES + 1).unwrap(),
        u32::MAX,
    ] {
        // The advertised value fits the backing, but exceeds the recovery ceiling.
        let memory = HeaderOnly {
            value_len,
            pages: 65_537,
        };
        assert!(matches!(
            decode_stable_cell_commit_store_from_memory(&memory),
            Err(StableCellLedgerError::Payload(StableCellPayloadError::TooLarge {
                value_len: rejected,
            })) if rejected == u64::from(value_len)
        ));
        // Physical-capacity rejection keeps precedence when both checks fail.
        let memory = HeaderOnly {
            value_len,
            pages: 1,
        };
        assert_eq!(
            decode_stable_cell_payload(&memory),
            Err(StableCellPayloadError::InvalidLength {
                value_len: u64::from(value_len),
                available_bytes: WASM_PAGE_SIZE_BYTES - STABLE_CELL_VALUE_OFFSET,
            })
        );
    }
    // The same header-only boundary admits a fixture's selected ceiling.
    for value_len in [16_385, u32::MAX] {
        let memory = HeaderOnly {
            value_len,
            pages: 65_537,
        };
        assert_eq!(
            stable_cell_payload_length(&memory, 16_384),
            Err(StableCellPayloadError::TooLarge {
                value_len: u64::from(value_len)
            }),
        );
    }
}

#[test]
fn hostile_wire_lengths_reject_on_production_record_decode() {
    use crate::encoding::{RECORD_LIMITS, RECORD_MAGIC};
    // Schema-correct snapshot prefix, then an unbacked payload length.
    let prefix = ic_wire::native::encode(&Some((1_u64, 0_u64, 0_u64)), RECORD_LIMITS).unwrap();
    for length in [
        crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1,
        usize::MAX,
    ] {
        let mut bytes = RECORD_MAGIC.to_vec();
        bytes.extend_from_slice(&prefix);
        bytes.extend(ic_wire::native::encode(&length, RECORD_LIMITS).unwrap());
        assert!(decode_stable_cell_commit_store(&bytes).is_err());
    }
    // A fully backed oversized opaque payload also fails its static bound.
    let payload = vec![0; crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1];
    let oversized =
        ic_wire::BytesRef::<{ crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1 }>::new(&payload)
            .unwrap();
    let mut bytes = RECORD_MAGIC.to_vec();
    bytes.extend(
        ic_wire::native::encode(&Some((1_u64, 0_u64, 0_u64, oversized)), RECORD_LIMITS).unwrap(),
    );
    assert!(decode_stable_cell_commit_store(&bytes).is_err());
    assert_eq!(
        decode_stable_cell_commit_store(&vec![0; crate::constants::MAX_LEDGER_RECORD_BYTES + 1]),
        Err(ic_wire::Error::Limit)
    );
}
