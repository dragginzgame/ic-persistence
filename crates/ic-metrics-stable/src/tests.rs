use std::{cell::Cell, rc::Rc};

use ic_memory::{
    Memory, MemoryManagerConfig, RuntimeMemory, RuntimeOpenError, VectorMemory,
    testing::fixture_runtime,
};
use ic_metrics::{
    MeasurementAggregate as Aggregate, MeasurementCounter as Counter,
    MeasurementHistogram as Histogram, MeasurementSummary as Summary,
};

use ic_storage::adapter::StableBTreeMap;

use crate::{MetricIdentity, MetricKey, MetricsStore, StoreError, codec};

// Isolated native host: declare a permanent key and commit allocation before opening.
fn admitted_memory<M: ic_memory::MemoryBacking>(backing: M) -> RuntimeMemory<M> {
    let runtime = fixture_runtime(
        backing,
        "metrics_test",
        "metrics.",
        &["metrics.store.v1"],
        MemoryManagerConfig::default(),
    )
    .unwrap();
    runtime.open_memory("metrics.store.v1").unwrap()
}

fn identity() -> MetricIdentity {
    MetricIdentity::new("instructions", 0).unwrap()
}
fn key(namespace: &str) -> MetricKey {
    MetricKey::new(namespace, "work").unwrap()
}
fn store() -> MetricsStore<VectorMemory> {
    MetricsStore::new(StableBTreeMap::new(
        admitted_memory(VectorMemory::default()),
    ))
}

fn encode<T: crate::metric::StoredMetric>(identity: &MetricIdentity, value: &T) -> Vec<u8> {
    codec::encode(identity, value).unwrap()
}

fn assert_rejected_record<T: crate::metric::StoredMetric + std::fmt::Debug>(
    store: &mut MetricsStore<WriteCountingMemory>,
    writes: &Cell<usize>,
    key: &MetricKey,
    identity: &MetricIdentity,
    value: &T,
    (bytes, error): (Vec<u8>, StoreError),
) {
    // Callers install the record before this check, including before reopening.
    writes.set(0);
    assert_eq!(
        store.load::<T>(key, identity, value.configuration()),
        Err(error)
    );
    assert_eq!(store.save(key, identity, value), Err(error));
    assert_eq!(store.map.get(&key.0), Some(bytes));
    assert_eq!(writes.get(), 0);
}

// The independent fixture uses plain tuples with the declared field order.
fn wire_state<T: serde::Serialize>(kind: u8, identity: &MetricIdentity, state: &T) -> Vec<u8> {
    let state = ic_wire::native::encode(state, fixture_limits()).unwrap();
    wire_row(kind, &identity.unit, identity.epoch, &state)
}

fn fixture_limits() -> ic_wire::Limits {
    ic_wire::Limits {
        bytes: codec::MAX_RECORD_BYTES,
        allocation: 1024 * 1024,
        nodes: 16 * 1024,
        depth: 16,
    }
}

fn wire_row(kind: u8, unit: &str, epoch: u64, state: &[u8]) -> Vec<u8> {
    let payload = ic_wire::BytesRef::<{ codec::MAX_RECORD_BYTES }>::new(state).unwrap();
    let encoded = ic_wire::native::encode(&(kind, unit, epoch, payload), fixture_limits()).unwrap();
    let mut bytes = b"ICMW".to_vec();
    bytes.extend_from_slice(&encoded);
    bytes
}

// Observe native Memory effects, not IC instructions, cycles or timings.
#[derive(Clone, Default)]
struct WriteCountingMemory {
    memory: VectorMemory,
    writes: Rc<Cell<usize>>,
}
impl ic_memory::MemoryBacking for WriteCountingMemory {
    fn backing_identity(&self) -> ic_memory::BackingIdentity {
        ic_memory::MemoryBacking::backing_identity(&self.memory)
    }
}

impl Memory for WriteCountingMemory {
    fn size(&self) -> u64 {
        self.memory.size()
    }

    fn grow(&self, pages: u64) -> i64 {
        self.memory.grow(pages)
    }

    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.memory.read(offset, dst);
    }

    fn write(&self, offset: u64, src: &[u8]) {
        self.writes.set(self.writes.get() + 1);
        self.memory.write(offset, src);
    }
}

fn assert_rejected_checkpoint<T: crate::metric::StoredMetric>(previous: &T, current: &T) {
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
    let key = key("append");
    store.save(&key, &identity(), previous).unwrap();
    let retained = store.map.get(&key.0);
    writes.set(0);
    assert_eq!(
        store.save(&key, &identity(), current),
        Err(StoreError::StateRegression)
    );
    assert_eq!(writes.get(), 0);
    assert_eq!(store.map.get(&key.0), retained);
}

// Mutate persisted bytes independently of the decoder's field/range formulas.
// A changed byte can be another valid state; admission is not corruption detection.
fn check_wire_mutations<T: crate::metric::StoredMetric + std::fmt::Debug>(
    value: &T,
) -> (usize, usize) {
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
    let key = key("mutation");
    let identity = identity();
    let original = encode(&identity, value);
    let mut admitted = 0;
    let mut refused = 0;
    for offset in 0..original.len() {
        for mask in [1, 2, 4, 8, 16, 32, 64, 128, 0x55, 0xaa, 0xff] {
            for paired in [false, true] {
                let mut bytes = original.clone();
                bytes[offset] ^= mask;
                if paired {
                    let adjacent = (offset + 1) % bytes.len();
                    bytes[adjacent] ^= mask;
                }
                store.map.insert(key.0.clone(), bytes.clone());
                writes.set(0);
                let loaded = store.load::<T>(&key, &identity, value.configuration());
                assert_eq!(writes.get(), 0);
                match loaded {
                    Err(error) => {
                        refused += 1;
                        assert_rejected_record(
                            &mut store,
                            &writes,
                            &key,
                            &identity,
                            value,
                            (bytes, error),
                        );
                    }
                    Ok(Some(restored)) => {
                        admitted += 1;
                        // Every admitted record must retain its exact canonical bytes.
                        assert_eq!(encode(&identity, &restored), bytes);
                        store.save(&key, &identity, &restored).unwrap();
                        assert_eq!(writes.get(), 0);
                        assert_eq!(store.map.get(&key.0), Some(bytes.clone()));
                        match store.save(&key, &identity, value) {
                            Ok(()) => {
                                assert_eq!(store.map.get(&key.0), Some(original.clone()));
                                assert_eq!(
                                    store
                                        .load::<T>(&key, &identity, value.configuration())
                                        .unwrap()
                                        .as_ref(),
                                    Some(value)
                                );
                            }
                            Err(error) => {
                                assert_eq!(error, StoreError::StateRegression);
                                assert_eq!(writes.get(), 0);
                                assert_eq!(store.map.get(&key.0), Some(bytes));
                            }
                        }
                    }
                    Ok(None) => panic!("installed mutation became an absent row"),
                }
            }
        }
    }
    (admitted, refused)
}

mod continuity;
mod encoding;
mod storage;
