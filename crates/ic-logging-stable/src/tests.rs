use std::{cell::Cell, num::NonZeroUsize, rc::Rc};

use ic_logging::{AppendError, EntryInput, Filter, Level, PageRequest};
use ic_memory::{
    MemoryBacking, MemoryManagerConfig, RuntimeOpenError, VectorMemory, testing::fixture_runtime,
};

use super::*;

fn limits() -> Limits {
    Limits {
        max_entries: NonZeroUsize::new(4).unwrap(),
        max_message_bytes: 64,
        max_label_bytes: 16,
    }
}
fn key() -> LogKey {
    LogKey::new("app", "worker").unwrap()
}
fn identity() -> LogIdentity {
    LogIdentity { epoch: 7 }
}
fn input(time: u64, message: &str) -> EntryInput<'_> {
    EntryInput {
        created_at_secs: time,
        level: Level::Info,
        source: "w",
        topic: None,
        message,
    }
}
fn admitted<M: MemoryBacking>(memory: M) -> ic_memory::RuntimeMemory<M> {
    fixture_runtime(
        memory,
        "logging_test",
        "logs.",
        &["logs.store.v1"],
        MemoryManagerConfig::default(),
    )
    .unwrap()
    .open_memory("logs.store.v1")
    .unwrap()
}
fn store() -> LoggingStore<VectorMemory> {
    LoggingStore::new(StableBTreeMap::new(admitted(VectorMemory::default())))
}

#[test]
fn governed_reopen_restores_the_same_engine_and_retains_the_lease() {
    let backing = VectorMemory::default();
    let runtime = fixture_runtime(
        backing,
        "logging_test",
        "logs.",
        &["logs.store.v1"],
        MemoryManagerConfig::default(),
    )
    .unwrap();
    let mut store = LoggingStore::new(StableBTreeMap::new(
        runtime.open_memory("logs.store.v1").unwrap(),
    ));
    assert!(matches!(
        runtime.open_memory("logs.store.v1"),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    assert!(store.load(&key(), identity(), limits()).unwrap().is_none());
    let mut log = Log::new(limits());
    for time in 10..17 {
        log.append(input(time, "é🦀".repeat(30).as_str())).unwrap();
    }
    store.save(&key(), identity(), &log).unwrap();
    let map = store.into_map();
    assert!(matches!(
        runtime.open_memory("logs.store.v1"),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    drop(map);
    let store = LoggingStore::new(StableBTreeMap::load(
        runtime.open_memory("logs.store.v1").unwrap(),
    ));
    let mut restored = store.load(&key(), identity(), limits()).unwrap().unwrap();
    assert_eq!(restored.snapshot(), log.snapshot());
    assert_eq!(restored.append(input(17, "resumed")), Ok(7));
    let page = restored.page(
        Filter::default(),
        PageRequest {
            before: None,
            max_results: NonZeroUsize::new(2).unwrap(),
            max_scanned: NonZeroUsize::new(2).unwrap(),
        },
    );
    assert_eq!(
        page.entries
            .iter()
            .map(|entry| entry.sequence)
            .collect::<Vec<_>>(),
        [7, 6]
    );
    assert_eq!(page.next_before, Some(6));
    assert!(page.entries[1].truncated);
}

#[test]
fn empty_and_exhausted_checkpoints_preserve_watermarks_across_reopen() {
    let backing = VectorMemory::default();
    let mut store = LoggingStore::new(StableBTreeMap::new(admitted(backing.clone())));
    let mut log = Log::new(limits());
    log.append(input(20, "expired")).unwrap();
    log.prune_expired(21, 0, 1);
    store.save(&key(), identity(), &log).unwrap();
    drop(store);
    let mut store = LoggingStore::new(StableBTreeMap::load(admitted(backing)));
    let mut log = store.load(&key(), identity(), limits()).unwrap().unwrap();
    assert!(log.is_empty());
    assert_eq!(
        log.append(input(19, "bad")),
        Err(AppendError::TimestampRegressed {
            previous: 20,
            current: 19
        })
    );
    assert_eq!(log.append(input(20, "next")), Ok(1));
    let exhausted = Log::restore(LogSnapshot {
        limits: limits(),
        entries: vec![],
        next_sequence: None,
        last_created_at_secs: Some(20),
    })
    .unwrap();
    store.save(&key(), identity(), &exhausted).unwrap();
    let mut restored = store.load(&key(), identity(), limits()).unwrap().unwrap();
    assert_eq!(
        restored.append(input(21, "bad")),
        Err(AppendError::SequenceExhausted)
    );
    assert_eq!(
        store.save(&key(), identity(), &log),
        Err(StoreError::StateRegression)
    );
}

#[derive(Clone, Default)]
struct CountedMemory {
    memory: VectorMemory,
    writes: Rc<Cell<usize>>,
}
impl MemoryBacking for CountedMemory {
    fn backing_identity(&self) -> ic_memory::BackingIdentity {
        self.memory.backing_identity()
    }
}
impl Memory for CountedMemory {
    fn size(&self) -> u64 {
        self.memory.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        self.memory.grow(pages)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        self.memory.read(offset, bytes);
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        self.writes.set(self.writes.get() + 1);
        self.memory.write(offset, bytes);
    }
}

#[test]
fn identical_and_refused_checkpoints_do_not_write_or_rewrite_issued_history() {
    let memory = CountedMemory::default();
    let writes = memory.writes.clone();
    let mut store = LoggingStore::new(StableBTreeMap::new(admitted(memory)));
    let mut log = Log::new(limits());
    for time in 10..13 {
        log.append(input(time, "event")).unwrap();
    }
    store.save(&key(), identity(), &log).unwrap();
    let bytes = store.map.get(&key().0);
    writes.set(0);
    store.save(&key(), identity(), &log).unwrap();
    assert_eq!(writes.get(), 0);
    for mode in 0..3 {
        let mut state = log.snapshot();
        match mode {
            0 => {
                state.entries.clear();
                state.next_sequence = Some(2);
            }
            1 => state.entries[0].message = "rewritten".into(),
            _ => {
                state.entries.clear();
                state.last_created_at_secs = Some(13);
            }
        }
        let invalid = Log::restore(state).unwrap();
        assert_eq!(
            store.save(&key(), identity(), &invalid),
            Err(StoreError::StateRegression)
        );
        assert_eq!(writes.get(), 0);
        assert_eq!(store.map.get(&key().0), bytes);
    }
    let original = log.snapshot();
    log.prune_expired(13, 0, 4);
    store.save(&key(), identity(), &log).unwrap();
    writes.set(0);
    let resurrected = Log::restore(original).unwrap();
    assert_eq!(
        store.save(&key(), identity(), &resurrected),
        Err(StoreError::StateRegression)
    );
    assert_eq!(writes.get(), 0);
    assert!(
        store
            .load(&key(), identity(), limits())
            .unwrap()
            .unwrap()
            .is_empty()
    );
}

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
    log.append(input(10, "m")).unwrap();
    let valid = codec::encode(identity(), &log.snapshot()).unwrap();
    assert_eq!(valid.len(), 75);
    for end in 0..valid.len() {
        assert!(codec::decode(&valid[..end], identity(), limits()).is_err());
    }
    for index in [24, 33, 63, 69] {
        let mut bytes = valid.clone();
        bytes[index] = 2;
        assert!(matches!(
            codec::decode(&bytes, identity(), limits()),
            Err(StoreError::MalformedRecord)
        ));
    }
    for index in [62, 68, 74] {
        let mut bytes = valid.clone();
        bytes[index] = 255;
        assert!(matches!(
            codec::decode(&bytes, identity(), limits()),
            Err(StoreError::MalformedRecord)
        ));
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
    let mut bytes = valid.clone();
    bytes[42..46].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::InvalidState(RestoreError::TooManyEntries))
    ));
    let mut bytes = valid.clone();
    bytes[64..68].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::MalformedRecord)
    ));
    let mut bytes = valid.clone();
    bytes[46..54].copy_from_slice(&1_u64.to_le_bytes());
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::InvalidState(RestoreError::InvalidSequence))
    ));
    let mut bytes = valid;
    bytes[63] = 1;
    assert!(matches!(
        codec::decode(&bytes, identity(), limits()),
        Err(StoreError::InvalidState(RestoreError::InvalidEntry))
    ));
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
fn explicit_removal_verifies_old_identity_and_keeps_keys_separate() {
    assert_eq!(LogKey::new("", "x"), Err(StoreError::InvalidIdentifier));
    assert_eq!(
        LogKey::new("app\0bad", "x"),
        Err(StoreError::InvalidIdentifier)
    );
    assert_eq!(
        LogKey::new("app", &"x".repeat(129)),
        Err(StoreError::InvalidIdentifier)
    );
    assert_ne!(LogKey::new("a", "b/c"), LogKey::new("a/b", "c"));
    let mut store = store();
    store.save(&key(), identity(), &Log::new(limits())).unwrap();
    assert_eq!(
        store.remove(&key(), LogIdentity { epoch: 8 }, limits()),
        Err(StoreError::IdentityMismatch)
    );
    assert_eq!(store.remove(&key(), identity(), limits()), Ok(true));
    assert_eq!(store.remove(&key(), identity(), limits()), Ok(false));
    store
        .save(&key(), LogIdentity { epoch: 8 }, &Log::new(limits()))
        .unwrap();
    assert!(
        store
            .load(&key(), LogIdentity { epoch: 8 }, limits())
            .unwrap()
            .is_some()
    );
}

#[test]
fn pruning_replaces_one_checkpoint_and_reuses_storage_after_warmup() {
    let mut store = store();
    let mut log = Log::new(limits());
    let mut baseline_pages = None;
    for turn in 0..32 {
        for offset in 0..4 {
            log.append(input(turn * 10 + offset, "retained")).unwrap();
        }
        store.save(&key(), identity(), &log).unwrap();
        log.prune_expired(turn * 10 + 5, 0, 4);
        store.save(&key(), identity(), &log).unwrap();
        assert_eq!(store.map.len(), 1);
        let (map, pages) = store.into_map().allocated_pages();
        store = LoggingStore::new(map);
        if turn == 8 {
            baseline_pages = Some(pages);
        }
        if turn > 8 {
            assert_eq!(Some(pages), baseline_pages);
        }
    }
    assert!(
        store
            .load(&key(), identity(), limits())
            .unwrap()
            .unwrap()
            .is_empty()
    );
}
