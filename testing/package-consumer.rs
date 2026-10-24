//! Independent archive consumer: shared lease identity, custody and reopen bytes.
use ic_memory::{MemoryManagerConfig, RuntimeOpenError, VectorMemory, testing::fixture_runtime};
use ic_metrics::MeasurementCounter;
use ic_metrics_stable::{MetricIdentity, MetricKey, MetricsStore};
use ic_storage::adapter::StableBTreeMap;

fn main() {
    let backing = VectorMemory::default();
    let open = || {
        fixture_runtime(
            backing.clone(),
            "archive",
            "archive.",
            &["archive.map.v1", "archive.metrics.v1"],
            MemoryManagerConfig::default(),
        )
        .unwrap()
    };
    {
        let runtime = open();
        let lease = runtime.open_memory("archive.map.v1").unwrap();
        let mut map = StableBTreeMap::<u64, u64, _>::new(lease);
        assert_eq!(map.insert(7, 11), None);
        assert!(matches!(
            runtime.open_memory("archive.map.v1"),
            Err(RuntimeOpenError::AlreadyOpen { .. })
        ));
        drop(map);
        assert!(runtime.open_memory("archive.map.v1").is_ok());
    }
    let runtime = open();
    let map = StableBTreeMap::<u64, u64, _>::load(runtime.open_memory("archive.map.v1").unwrap());
    assert_eq!(map.get(&7), Some(11));

    let key = MetricKey::new("archive", "bytes").unwrap();
    let identity = MetricIdentity::new("bytes", 0).unwrap();
    let mut counter = MeasurementCounter::ZERO;
    counter.add(42);
    let mut store = MetricsStore::new(StableBTreeMap::new(
        runtime.open_memory("archive.metrics.v1").unwrap(),
    ));
    store.save_counter(&key, &identity, &counter).unwrap();
    assert!(matches!(
        runtime.open_memory("archive.metrics.v1"),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    drop(store.into_map());
    let reopened = MetricsStore::new(StableBTreeMap::load(
        runtime.open_memory("archive.metrics.v1").unwrap(),
    ));
    assert_eq!(
        reopened.load_counter(&key, &identity).unwrap(),
        Some(counter)
    );
}
