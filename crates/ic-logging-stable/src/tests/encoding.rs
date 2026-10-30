//! encoding contracts.
use super::*;

#[test]
fn record_budget_identity_and_configuration_refusals_preserve_retained_bytes() {
    let memory = CountedMemory::default();
    let writes = memory.writes.clone();
    let mut store = LoggingStore::new(StableBTreeMap::new(admitted(memory)));
    let limits = Limits {
        max_entries: NonZeroUsize::new(32).unwrap(),
        max_message_bytes: 4096,
        ..limits()
    };
    let mut log = Log::new(limits);
    store.save(&key(), identity(), &log).unwrap();
    let bytes = store.map.get(&key().0);
    writes.set(0);
    let wrong = LogIdentity { epoch: 8 };
    assert!(matches!(
        store.load(&key(), wrong, limits),
        Err(StoreError::IdentityMismatch)
    ));
    assert_eq!(
        store.save(&key(), wrong, &log),
        Err(StoreError::IdentityMismatch)
    );
    let other = Limits {
        max_message_bytes: 1,
        ..limits
    };
    assert!(matches!(
        store.load(&key(), identity(), other),
        Err(StoreError::LimitsMismatch)
    ));
    for time in 0..20 {
        log.append(input(time, &"x".repeat(4096))).unwrap();
    }
    assert_eq!(
        store.save(&key(), identity(), &log),
        Err(StoreError::RecordTooLarge)
    );
    let unsupported = Limits {
        max_entries: NonZeroUsize::new(MAX_ENTRIES + 1).unwrap(),
        ..limits
    };
    assert!(matches!(
        store.load(&key(), identity(), unsupported),
        Err(StoreError::UnsupportedLimits)
    ));
    assert_eq!(
        store.save(&key(), identity(), &Log::new(unsupported)),
        Err(StoreError::UnsupportedLimits)
    );
    assert_eq!(store.map.get(&key().0), bytes);
    assert_eq!(writes.get(), 0);
}

#[test]
fn malformed_bytes_are_never_empty_or_reset_fallbacks() {
    let mut log = Log::new(limits());
    log.append(input(10, "utf8-message")).unwrap();
    let valid = codec::encode(identity(), &log.snapshot()).unwrap();
    assert_eq!(
        valid,
        b"ICL\x02\x07\x04\x40\x10\x01\x01\x01\x0a\x01\x00\x0a\x01\x00\x01w\x00\x0cutf8-message"
    );
    for end in 0..valid.len() {
        assert!(codec::decode(&valid[..end], identity(), limits()).is_err());
    }
    let mut bytes = valid.clone();
    bytes.push(0);
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::MalformedRecord)
    ));
    let mut bytes = valid.clone();
    bytes[3] = 255;
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::UnsupportedFormat)
    ));
    let message = valid
        .windows(b"utf8-message".len())
        .position(|bytes| bytes == b"utf8-message")
        .unwrap();
    let mut bytes = valid;
    bytes[message] = 255;
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::MalformedRecord)
    ));

    let mut state = log.snapshot();
    state.entries[0].sequence = 1;
    let bytes = codec::encode(identity(), &state).unwrap();
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::InvalidState(RestoreError::InvalidSequence))
    ));
    state = log.snapshot();
    state.entries[0].truncated = true;
    let bytes = codec::encode(identity(), &state).unwrap();
    let mut store = store();
    store.map.insert(key().0, bytes.clone());
    assert_eq!(
        store.save(&key(), identity(), &log),
        Err(StoreError::InvalidState(RestoreError::InvalidEntry))
    );
    assert_eq!(store.map.get(&key().0), Some(bytes));
    assert!(matches!(
        codec::decode(&vec![0; MAX_RECORD_BYTES + 1], identity(), limits()),
        Err(StoreError::RecordTooLarge)
    ));
}

#[test]
fn wire_checkpoint_admits_configured_fields_before_owned_restoration() {
    let mut log = Log::new(limits());
    log.append(input(10, "event")).unwrap();
    for field in 0..3 {
        let mut state = log.snapshot();
        match field {
            0 => state.entries[0].source = "s".repeat(limits().max_label_bytes + 1),
            1 => state.entries[0].topic = Some("t".repeat(limits().max_label_bytes + 1)),
            _ => state.entries[0].message = "m".repeat(limits().max_message_bytes + 1),
        }
        let bytes = codec::encode(identity(), &state).unwrap();
        assert!(matches!(
            codec::decode(&bytes, identity(), limits()),
            Err(StoreError::MalformedRecord)
        ));
    }
    let mut state = log.snapshot();
    state.entries = vec![state.entries[0].clone(); limits().max_entries.get() + 1];
    let bytes = codec::encode(identity(), &state).unwrap();
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::InvalidState(RestoreError::TooManyEntries))
    ));
}

#[test]
fn wire_budgets_admit_maximum_rows_and_large_valid_checkpoints() {
    for (count, message_bytes) in [(MAX_ENTRIES, 0), (14, MAX_MESSAGE_BYTES)] {
        let limits = Limits {
            max_entries: NonZeroUsize::new(MAX_ENTRIES).unwrap(),
            max_message_bytes: MAX_MESSAGE_BYTES,
            max_label_bytes: MAX_LABEL_BYTES,
        };
        let message = "m".repeat(message_bytes);
        let label = "s".repeat(if count == MAX_ENTRIES {
            1
        } else {
            MAX_LABEL_BYTES
        });
        let mut log = Log::new(limits);
        for _ in 0..count {
            log.append(EntryInput {
                created_at_secs: u64::MAX,
                level: Level::Error,
                source: &label,
                topic: Some(&label),
                message: &message,
            })
            .unwrap();
        }
        let bytes = codec::encode(identity(), &log.snapshot()).unwrap_or_else(|error| {
            panic!("{count} rows with {message_bytes} message bytes: {error}")
        });
        assert!(bytes.len() <= MAX_RECORD_BYTES);
        assert_eq!(
            codec::decode(&bytes, identity(), limits)
                .unwrap()
                .snapshot(),
            log.snapshot()
        );
    }
}
