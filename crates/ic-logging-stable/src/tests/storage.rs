//! storage contracts.
use super::*;

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
    assert_eq!(
        restored
            .append(input(17, "resumed"))
            .map(|receipt| receipt.sequence),
        Ok(7)
    );
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
