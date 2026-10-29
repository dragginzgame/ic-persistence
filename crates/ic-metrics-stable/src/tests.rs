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

#[test]
fn store_error_retains_the_arithmetic_source() {
    use std::error::Error as _;
    let state = ic_metrics::MeasurementStateError::InvalidBounds;
    let error = StoreError::from(state);
    assert_eq!(error, StoreError::InvalidState(state));
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<ic_metrics::MeasurementStateError>(),
        Some(&state)
    );
    assert!(StoreError::StateRegression.source().is_none());
}

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

#[test]
fn store_retains_allocation_lease_through_map_transfer() {
    let runtime = fixture_runtime(
        VectorMemory::default(),
        "metrics_test",
        "metrics.",
        &["metrics.store.v1"],
        MemoryManagerConfig::default(),
    )
    .unwrap();
    let mut store = MetricsStore::new(StableBTreeMap::new(
        runtime.open_memory("metrics.store.v1").unwrap(),
    ));
    store
        .save_counter(&key("app"), &identity(), &Counter::ZERO)
        .unwrap();
    assert!(matches!(
        runtime.open_memory("metrics.store.v1"),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    let map: crate::MetricsMap<_> = store.into_map();
    assert!(matches!(
        runtime.open_memory("metrics.store.v1"),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    drop(map);
    let reopened = MetricsStore::new(StableBTreeMap::load(
        runtime.open_memory("metrics.store.v1").unwrap(),
    ));
    assert!(matches!(
        runtime.open_memory("metrics.store.v1"),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    assert_eq!(
        reopened.load_counter(&key("app"), &identity()),
        Ok(Some(Counter::ZERO))
    );
}

#[test]
fn frozen_wire_rows_reopen_through_the_governed_map() {
    let backing = VectorMemory::default();
    let mut records = StableBTreeMap::<Vec<u8>, Vec<u8>, _>::new(admitted_memory(backing.clone()));
    let id = MetricIdentity::new("x", 3).unwrap();
    // Frozen empty Wire states, independent of the production encoder.
    for (name, kind, payload) in [
        ("aggregate", 1, 5),
        ("summary", 2, 6),
        ("histogram", 3, 8),
        ("counter", 4, 2),
    ] {
        let key = MetricKey::new("app", name).unwrap();
        let mut bytes = b"ICMW".to_vec();
        bytes.extend_from_slice(&[kind, 1, b'x', 3, payload]);
        bytes.resize(bytes.len() + usize::from(payload), 0);
        records.insert(key.0, bytes);
    }
    let before: Vec<_> = records
        .iter()
        .map(|entry| (entry.key().clone(), entry.value()))
        .collect();
    drop(records);
    let store = MetricsStore::new(StableBTreeMap::load(admitted_memory(backing.clone())));
    let key = |name| MetricKey::new("app", name).unwrap();
    assert_eq!(
        store.load_aggregate(&key("aggregate"), &id),
        Ok(Some(Aggregate::EMPTY))
    );
    assert_eq!(
        store.load_summary(&key("summary"), &id),
        Ok(Some(Summary::EMPTY))
    );
    assert_eq!(
        store.load_histogram(&key("histogram"), &id, &[]),
        Ok(Some(Histogram::new([]).unwrap()))
    );
    assert_eq!(
        store.load_counter(&key("counter"), &id),
        Ok(Some(Counter::ZERO))
    );
    assert_eq!(
        store
            .into_map()
            .iter()
            .map(|entry| (entry.key().clone(), entry.value()))
            .collect::<Vec<_>>(),
        before
    );
    let mut store = MetricsStore::new(StableBTreeMap::load(admitted_memory(backing.clone())));
    store
        .save_counter(
            &key("counter"),
            &id,
            &Counter::from_parts(9, false).unwrap(),
        )
        .unwrap();
    drop(store);
    let records = StableBTreeMap::<Vec<u8>, Vec<u8>, _>::load(admitted_memory(backing));
    let bytes = records.get(&key("counter").0).unwrap();
    assert_eq!(bytes, b"ICMW\x04\x01x\x03\x02\x09\x00");
}

#[test]
fn stored_kind_admission_rejects_every_unknown_tag_before_identity() {
    let id = identity();
    let mut bytes = encode(&id, &Counter::ZERO);
    for kind in 0..=u8::MAX {
        bytes[4] = kind;
        let result = codec::envelope(&bytes, &id);
        assert_eq!(result.is_ok(), (1..=4).contains(&kind));
        if !(1..=4).contains(&kind) {
            assert!(matches!(result, Err(StoreError::KindMismatch)));
        }
    }
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

#[test]
fn truncated_records_reject_all_metric_families_without_writes() {
    fn check<T: crate::metric::StoredMetric + std::fmt::Debug>(value: &T) {
        let memory = WriteCountingMemory::default();
        let writes = memory.writes.clone();
        let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
        let key = key("app");
        let identity = identity();
        let bytes = encode(&identity, value);
        for length in 0..bytes.len() {
            let truncated = bytes[..length].to_vec();
            store.map.insert(key.0.clone(), truncated.clone());
            assert_rejected_record(
                &mut store,
                &writes,
                &key,
                &identity,
                value,
                (truncated, StoreError::MalformedRecord),
            );
        }
    }

    check(&Aggregate::EMPTY);
    let mut aggregate = Aggregate::EMPTY;
    aggregate.record(7);
    check(&aggregate);
    check(&Summary::EMPTY);
    check(&Summary::from_parts(aggregate, Some(7)).unwrap());
    check(&Counter::ZERO);
    check(&Counter::from_parts(7, false).unwrap());
    let mut histogram = Histogram::new([0, 10]).unwrap();
    check(&histogram);
    histogram.record(5);
    check(&histogram);
    check(&Histogram::new([]).unwrap());
}

#[test]
fn stored_unit_bytes_preserve_malformed_and_identity_errors() {
    let identity = MetricIdentity::new("x", 3).unwrap();
    let encoded = encode(&identity, &Aggregate::EMPTY);
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
    let key = key("app");
    for byte in 0..=u8::MAX {
        let mut bytes = encoded.clone();
        bytes[6] = byte;
        if byte == b'x' {
            assert_eq!(
                codec::decode::<Aggregate>(&bytes, &identity, &()),
                Ok(Aggregate::EMPTY)
            );
            continue;
        }
        let error = if byte.is_ascii_alphanumeric() || b"._/-:".contains(&byte) {
            StoreError::IdentityMismatch
        } else {
            StoreError::MalformedRecord
        };
        store.map.insert(key.0.clone(), bytes.clone());
        assert_rejected_record(
            &mut store,
            &writes,
            &key,
            &identity,
            &Aggregate::EMPTY,
            (bytes, error),
        );
    }
}

#[test]
fn one_map_reopens_mixed_types_and_namespaces_with_wide_totals() {
    let memory = VectorMemory::default();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory.clone())));
    let mut aggregate = Aggregate::EMPTY;
    let mut summary = Summary::EMPTY;
    let mut histogram = Histogram::new([0, 10]).unwrap();
    for value in [0, u64::MAX, u64::MAX, 9] {
        aggregate.record(value);
        summary.record(value);
        histogram.record(value);
    }
    store
        .save_aggregate(&key("icydb"), &identity(), &aggregate)
        .unwrap();
    store
        .save_summary(&key("canic"), &identity(), &summary)
        .unwrap();
    store
        .save_histogram(&key("timers"), &identity(), &histogram)
        .unwrap();
    let bytes_identity = MetricIdentity::new("bytes", 0).unwrap();
    let counter = Counter::from_parts(u128::from(u64::MAX) + 42, false).unwrap();
    store
        .save_counter(&key("backup"), &bytes_identity, &counter)
        .unwrap();
    drop(store);
    let reopened = MetricsStore::new(StableBTreeMap::load(admitted_memory(memory)));
    assert_eq!(
        reopened.load_aggregate(&key("icydb"), &identity()),
        Ok(Some(aggregate))
    );
    assert_eq!(
        reopened.load_summary(&key("canic"), &identity()),
        Ok(Some(summary))
    );
    assert_eq!(
        reopened.load_histogram(&key("timers"), &identity(), &[0, 10]),
        Ok(Some(histogram))
    );
    assert_eq!(
        reopened.load_counter(&key("backup"), &bytes_identity),
        Ok(Some(counter))
    );
    assert_eq!(
        reopened.load_aggregate(&key("missing"), &identity()),
        Ok(None)
    );
    assert!(aggregate.total() > u128::from(u64::MAX));
}

#[test]
fn identity_kind_bounds_and_regression_reject_without_mutation() {
    let mut store = store();
    let mut histogram = Histogram::new([10]).unwrap();
    histogram.record(7);
    let key = key("app");
    store.save(&key, &identity(), &histogram).unwrap();
    assert_eq!(
        store.save(&key, &MetricIdentity::new("bytes", 0).unwrap(), &histogram),
        Err(StoreError::IdentityMismatch)
    );
    assert_eq!(
        store.save(
            &key,
            &MetricIdentity::new("instructions", 1).unwrap(),
            &histogram
        ),
        Err(StoreError::IdentityMismatch)
    );
    assert_eq!(
        store.save(&key, &identity(), &Aggregate::EMPTY),
        Err(StoreError::KindMismatch)
    );
    assert_eq!(
        store.save(&key, &identity(), &Histogram::new([11]).unwrap()),
        Err(StoreError::BoundsMismatch)
    );
    assert_eq!(
        store.save(&key, &identity(), &Histogram::new([10]).unwrap()),
        Err(StoreError::StateRegression)
    );
    assert_eq!(
        store.load_histogram(&key, &identity(), histogram.upper_bounds()),
        Ok(Some(histogram))
    );
    assert!(store.remove(&key, &identity()).unwrap());
    let next = MetricIdentity::new("instructions", 1).unwrap();
    store.save(&key, &next, &Aggregate::EMPTY).unwrap();
    assert_eq!(
        store.load_aggregate(&key, &next),
        Ok(Some(Aggregate::EMPTY))
    );
}

#[test]
fn exact_caps_and_overflow_flags_survive_storage_and_continue() {
    let mut store = store();
    let key = key("app");
    let mut value = Aggregate::from_parts(u64::MAX, 0, Some(0), false, false).unwrap();
    store.save(&key, &identity(), &value).unwrap();
    let exact = store.load_aggregate(&key, &identity()).unwrap().unwrap();
    assert!(!exact.samples_overflowed());
    value.record(0);
    store.save(&key, &identity(), &value).unwrap();
    assert_eq!(
        store.save(&key, &identity(), &exact),
        Err(StoreError::StateRegression)
    );
    assert_eq!(store.load_aggregate(&key, &identity()), Ok(Some(value)));
    let wide = Aggregate::from_parts(u64::MAX, u128::MAX, Some(u64::MAX), true, true).unwrap();
    store.save(&key, &identity(), &wide).unwrap();
    assert_eq!(store.load_aggregate(&key, &identity()), Ok(Some(wide)));
    let histogram = Histogram::from_parts([0], [u64::MAX], 0, [true], false, value).unwrap();
    store
        .save(
            &crate::MetricKey::new("app", "distribution").unwrap(),
            &identity(),
            &histogram,
        )
        .unwrap();
}

#[test]
fn aggregate_wire_envelope_errors_and_explicit_removal() {
    let mut store = store();
    let key = key("app");
    let bytes = encode(&identity(), &Aggregate::EMPTY);
    let mut unknown = bytes.clone();
    unknown[3] = 99;
    assert_eq!(
        codec::decode::<Aggregate>(&unknown, &identity(), &()),
        Err(StoreError::UnsupportedFormat)
    );
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        codec::decode::<Aggregate>(&trailing, &identity(), &()),
        Err(StoreError::MalformedRecord)
    );
    let mut flags = bytes;
    *flags.last_mut().unwrap() = 4;
    assert_eq!(
        codec::decode::<Aggregate>(&flags, &identity(), &()),
        Err(StoreError::MalformedRecord)
    );
    store
        .map
        .insert(key.0.clone(), wire_row(1, "instructions", 0, &[4]));
    assert!(store.remove(&key, &identity()).unwrap());
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

#[test]
fn wire_encoding_is_explicit_and_rejects_invalid_numeric_state() {
    let mut value = Aggregate::EMPTY;
    value.record(7);
    let identity = MetricIdentity::new("x", 3).unwrap();
    let aggregate = (1_u64, 7_u128, Some(7_u64), false, false);
    let expected = wire_state(1, &identity, &aggregate);
    assert_eq!(encode(&identity, &value), expected);
    assert_eq!(expected, b"ICMW\x01\x01x\x03\x06\x01\x07\x01\x07\x00\x00");
    let summary = Summary::from_parts(value, Some(7)).unwrap();
    assert_eq!(
        encode(&identity, &summary),
        wire_state(2, &identity, &(aggregate, Some(7_u64)))
    );
    let mut histogram = Histogram::new([10]).unwrap();
    histogram.record(7);
    assert_eq!(
        encode(&identity, &histogram),
        wire_state(
            3,
            &identity,
            &(aggregate, vec![(10_u64, 1_u64, false)], 0_u64, false)
        )
    );
    let invalid = wire_state(1, &identity, &(1_u64, 0_u128, Some(7_u64), false, false));
    assert!(matches!(
        codec::decode::<Aggregate>(&invalid, &identity, &()),
        Err(StoreError::InvalidState(_))
    ));
}

#[test]
fn counter_checkpoints_preserve_wide_caps_identity_and_explicit_resets() {
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory.clone())));
    let key = key("app");
    let identity = MetricIdentity::new("bytes", 7).unwrap();
    assert_eq!(store.load_counter(&key, &identity), Ok(None));
    for value in [
        Counter::ZERO,
        Counter::from_parts(u128::from(u64::MAX) + 1, false).unwrap(),
        Counter::from_parts(u128::MAX, false).unwrap(),
        Counter::from_parts(u128::MAX, true).unwrap(),
    ] {
        store.save(&key, &identity, &value).unwrap();
        assert_eq!(store.load_counter(&key, &identity), Ok(Some(value)));
    }
    drop(store);
    let mut store = MetricsStore::new(StableBTreeMap::load(admitted_memory(memory)));
    let retained = Counter::from_parts(u128::MAX, true).unwrap();
    assert_eq!(store.load_counter(&key, &identity), Ok(Some(retained)));
    writes.set(0);
    for regression in [
        Counter::ZERO,
        Counter::from_parts(u128::MAX, false).unwrap(),
    ] {
        assert_eq!(
            store.save(&key, &identity, &regression),
            Err(StoreError::StateRegression)
        );
    }
    for wrong in [
        MetricIdentity::new("items", 7).unwrap(),
        MetricIdentity::new("bytes", 8).unwrap(),
    ] {
        assert_eq!(
            store.load_counter(&key, &wrong),
            Err(StoreError::IdentityMismatch)
        );
        assert_eq!(
            store.save(&key, &wrong, &retained),
            Err(StoreError::IdentityMismatch)
        );
        assert_eq!(
            store.remove(&key, &wrong),
            Err(StoreError::IdentityMismatch)
        );
    }
    assert_eq!(
        store.load_aggregate(&key, &identity),
        Err(StoreError::KindMismatch)
    );
    assert_eq!(
        store.save(&key, &identity, &Aggregate::EMPTY),
        Err(StoreError::KindMismatch)
    );
    assert_eq!(writes.get(), 0);
    assert_eq!(store.load_counter(&key, &identity), Ok(Some(retained)));
    assert!(store.remove(&key, &identity).unwrap());
    let next = MetricIdentity::new("bytes", 8).unwrap();
    store.save(&key, &next, &Counter::ZERO).unwrap();
    assert_eq!(store.load_counter(&key, &next), Ok(Some(Counter::ZERO)));
    assert_eq!(
        store.load_counter(&key, &identity),
        Err(StoreError::IdentityMismatch)
    );
}

#[test]
fn counter_wire_records_reject_corruption_before_checkpoint_or_reset() {
    let value = Counter::from_parts(7, false).unwrap();
    let identity = MetricIdentity::new("x", 3).unwrap();
    let bytes = encode(&identity, &value);
    let expected = b"ICMW\x04\x01x\x03\x02\x07\x00".to_vec();
    assert_eq!(bytes, expected);
    let mut malformed = Vec::new();
    let mut trailing = bytes.clone();
    trailing.push(0);
    malformed.push((trailing, StoreError::MalformedRecord));
    let mut invalid_flag = bytes.clone();
    *invalid_flag.last_mut().unwrap() = 2;
    malformed.push((invalid_flag, StoreError::MalformedRecord));
    let mut impossible_overflow = bytes.clone();
    *impossible_overflow.last_mut().unwrap() = 1;
    malformed.push((
        impossible_overflow,
        StoreError::InvalidState(ic_metrics::MeasurementStateError::InvalidOverflowFlag),
    ));
    let mut unknown_kind = bytes.clone();
    unknown_kind[4] = 5;
    malformed.push((unknown_kind, StoreError::KindMismatch));
    let mut unknown_format = bytes;
    unknown_format[3] = 2;
    malformed.push((unknown_format, StoreError::UnsupportedFormat));
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
    let key = key("app");
    for record in malformed {
        store.map.insert(key.0.clone(), record.0.clone());
        assert_rejected_record(&mut store, &writes, &key, &identity, &value, record);
    }
    assert_eq!(
        store.remove(&key, &identity),
        Err(StoreError::UnsupportedFormat)
    );
    assert_eq!(writes.get(), 0);
    // Explicit disposition can retire a known envelope with invalid payload.
    store
        .map
        .insert(key.0.clone(), wire_row(4, "x", 3, &[7, 2]));
    assert!(store.remove(&key, &identity).unwrap());
    assert!(!store.remove(&key, &identity).unwrap());
    // An aggregate's existing kind cannot be reinterpreted as a scalar counter.
    store.save(&key, &identity, &Aggregate::EMPTY).unwrap();
    writes.set(0);
    assert_eq!(
        store.load_counter(&key, &identity),
        Err(StoreError::KindMismatch)
    );
    assert_eq!(
        store.save(&key, &identity, &value),
        Err(StoreError::KindMismatch)
    );
    assert_eq!(writes.get(), 0);
}

#[test]
fn inconsistent_summary_latest_rejects_load_and_save_without_overwrite() {
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory.clone())));
    let key = key("app");
    let identity = identity();
    let mut summary = Summary::EMPTY;
    summary.record(10);
    summary.record(10);
    let mut bytes = encode(&identity, &summary);
    // A present zero latest contradicts two tens.
    *bytes.last_mut().unwrap() = 0;
    store.map.insert(key.0.clone(), bytes.clone());
    drop(store);
    let mut reopened = MetricsStore::new(StableBTreeMap::load(admitted_memory(memory)));
    assert_rejected_record(
        &mut reopened,
        &writes,
        &key,
        &identity,
        &summary,
        (
            bytes,
            StoreError::InvalidState(ic_metrics::MeasurementStateError::InvalidLatest),
        ),
    );
}

#[test]
fn bounded_inputs_empty_zero_and_idempotent_checkpoints() {
    assert_eq!(MetricKey::new("", "x"), Err(StoreError::InvalidIdentifier));
    assert_eq!(
        MetricKey::new("a\0b", "x"),
        Err(StoreError::InvalidIdentifier)
    );
    assert_eq!(
        MetricKey::new("a", &"x".repeat(129)),
        Err(StoreError::InvalidIdentifier)
    );
    assert_eq!(
        MetricIdentity::new("µs", 0),
        Err(StoreError::InvalidIdentifier)
    );
    assert_ne!(MetricKey::new("a", "b/c"), MetricKey::new("a/b", "c"));
    let mut store = store();
    let key = key("app");
    let mut value = Summary::EMPTY;
    store.save(&key, &identity(), &value).unwrap();
    assert_eq!(
        store.load_summary(&key, &identity()),
        Ok(Some(Summary::EMPTY))
    );
    value.record(0);
    store.save(&key, &identity(), &value).unwrap();
    store.save(&key, &identity(), &value).unwrap();
    assert_eq!(
        store
            .load_summary(&key, &identity())
            .unwrap()
            .unwrap()
            .latest(),
        Some(0)
    );
    let bounds = std::array::from_fn::<_, 257, _>(|index| u64::try_from(index).unwrap());
    let unsupported = Histogram::new(bounds).unwrap();
    let oversized_key = crate::MetricKey::new("app", "oversize").unwrap();
    assert_eq!(
        store.load_histogram(&oversized_key, &identity(), unsupported.upper_bounds()),
        Err(StoreError::TooManyBounds)
    );
    assert_eq!(
        store.save_histogram(&oversized_key, &identity(), &unsupported),
        Err(StoreError::TooManyBounds)
    );
    assert!(store.map.get(&oversized_key.0).is_none());
    store.map.insert(oversized_key.0.clone(), vec![0]);
    assert_eq!(
        store.load_histogram(&oversized_key, &identity(), unsupported.upper_bounds()),
        Err(StoreError::TooManyBounds)
    );
    assert_eq!(
        store.save_histogram(&oversized_key, &identity(), &unsupported),
        Err(StoreError::TooManyBounds)
    );
    assert_eq!(store.map.get(&oversized_key.0), Some(vec![0]));
    let bytes = vec![0; codec::MAX_RECORD_BYTES + 1];
    assert_eq!(
        codec::decode::<Aggregate>(&bytes, &identity(), &()),
        Err(StoreError::RecordTooLarge)
    );
}

#[test]
fn histogram_checkpoints_cover_maximum_and_zero_bound_counts() {
    let identity = identity();
    let bounds = std::array::from_fn::<_, 256, _>(|index| u64::try_from(index).unwrap());
    let mut value = Histogram::new(bounds).unwrap();
    for observation in [0, 127, 255, 256] {
        value.record(observation);
    }
    let bytes = encode(&identity, &value);
    assert_eq!(
        codec::decode::<Histogram<256>>(&bytes, &identity, value.upper_bounds()),
        Ok(value)
    );
    let mut zero = Histogram::new([]).unwrap();
    zero.record(0);
    let bytes = encode(&identity, &zero);
    assert_eq!(
        codec::decode::<Histogram<0>>(&bytes, &identity, zero.upper_bounds()),
        Ok(zero)
    );
}

#[test]
fn checkpoints_reject_same_count_rewrites_and_admit_order_after_overflow() {
    let mut store = store();
    let key = key("app");
    let initial = Aggregate::from_parts(2, 8, Some(5), false, false).unwrap();
    store.save(&key, &identity(), &initial).unwrap();
    let rewrite = Aggregate::from_parts(2, 9, Some(5), false, false).unwrap();
    assert_eq!(
        store.save(&key, &identity(), &rewrite),
        Err(StoreError::StateRegression)
    );
    let summary_key = MetricKey::new("app", "ordered").unwrap();
    let aggregate = Aggregate::from_parts(u64::MAX, u128::MAX, Some(u64::MAX), true, true).unwrap();
    let mut summary = Summary::from_parts(aggregate, Some(2)).unwrap();
    store.save(&summary_key, &identity(), &summary).unwrap();
    summary.record(3);
    store.save(&summary_key, &identity(), &summary).unwrap();
    assert_eq!(
        store.load_summary(&summary_key, &identity()),
        Ok(Some(summary))
    );
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

#[test]
fn checkpoints_reject_impossible_exact_appends_without_writes() {
    // Only 1 was added to the total, which cannot introduce maximum 3.
    assert_rejected_checkpoint(
        &Aggregate::from_parts(2, 2, Some(1), false, false).unwrap(),
        &Aggregate::from_parts(3, 3, Some(3), false, false).unwrap(),
    );
    // One added observation cannot exceed the new maximum of 1.
    assert_rejected_checkpoint(
        &Aggregate::from_parts(1, 0, Some(0), false, false).unwrap(),
        &Aggregate::from_parts(2, 2, Some(1), false, false).unwrap(),
    );
    let summary = |samples: &[u64]| {
        let mut value = Summary::EMPTY;
        for sample in samples {
            value.record(*sample);
        }
        value
    };
    for (previous, current) in [
        // One addition must equal the latest, not just fall below the maximum.
        (summary(&[2, 1]), summary(&[1, 1, 2])),
        (summary(&[2, 0]), summary(&[1, 2, 1])),
        // A distinct new maximum and latest must both fit in the added total.
        (summary(&[1]), summary(&[0, 0, 3, 1])),
    ] {
        assert_rejected_checkpoint(&previous, &current);
    }
    let histogram = |samples: &[u64]| {
        let mut value = Histogram::new([1, 10]).unwrap();
        for sample in samples {
            value.record(*sample);
        }
        value
    };
    for (previous, current) in [
        // An addition in (1, 10] cannot add just 1 to the total.
        (histogram(&[1, 1, 3]), histogram(&[0, 1, 2, 3])),
        // An addition in [0, 1] cannot add 2 to the total.
        (histogram(&[0, 3]), histogram(&[1, 1, 3])),
        // The bucket containing a new maximum must have an added observation.
        (histogram(&[0, 5]), histogram(&[1, 1, 1, 1, 1, 0, 0, 6])),
        // Added bucket ranges must allow actually attaining the new maximum.
        (histogram(&[0, 5]), histogram(&[0, 2, 3, 9])),
    ] {
        assert_rejected_checkpoint(&previous, &current);
    }
}

#[test]
fn checkpoint_continuity_admits_bounded_recorded_extensions() {
    use crate::metric::StoredMetric;

    let observations = [0, 1, 3, u64::MAX];
    for prefix_length in 0..=3 {
        for mut prefix in 0..4_usize.pow(prefix_length) {
            let mut previous = Summary::EMPTY;
            let mut previous_histogram = Histogram::new([1, 10]).unwrap();
            for _ in 0..prefix_length {
                let value = observations[prefix % 4];
                previous.record(value);
                previous_histogram.record(value);
                prefix /= 4;
            }
            for added_length in 0..=3 {
                for mut added in 0..4_usize.pow(added_length) {
                    let mut current = previous;
                    let mut histogram = previous_histogram;
                    for _ in 0..added_length {
                        let value = observations[added % 4];
                        current.record(value);
                        histogram.record(value);
                        added /= 4;
                    }
                    assert!(current.aggregate().continues(&previous.aggregate()));
                    assert!(current.continues(&previous));
                    assert!(histogram.continues(&previous_histogram));
                }
            }
        }
    }
}

#[test]
fn checkpoint_continuity_preserves_exact_caps_and_clipped_appends() {
    use crate::metric::StoredMetric;

    let maximum = u64::MAX;
    let mut aggregate = Aggregate::from_parts(
        maximum - 1,
        u128::from(maximum - 1) * u128::from(maximum),
        Some(maximum),
        false,
        false,
    )
    .unwrap();
    let mut summary = Summary::from_parts(aggregate, Some(maximum)).unwrap();
    let mut histogram = Histogram::from_parts(
        [0, maximum],
        [0, maximum - 1],
        0,
        [false; 2],
        false,
        aggregate,
    )
    .unwrap();
    let mut store = store();
    for value in [maximum, 0, maximum, maximum, 1] {
        let previous = (aggregate, summary, histogram);
        aggregate.record(value);
        summary.record(value);
        histogram.record(value);
        assert!(aggregate.continues(&previous.0));
        assert!(summary.continues(&previous.1));
        assert!(histogram.continues(&previous.2));
        store
            .save_aggregate(&key("aggregate"), &identity(), &previous.0)
            .unwrap();
        store
            .save_aggregate(&key("aggregate"), &identity(), &aggregate)
            .unwrap();
        store
            .save_summary(&key("summary"), &identity(), &previous.1)
            .unwrap();
        store
            .save_summary(&key("summary"), &identity(), &summary)
            .unwrap();
        store
            .save_histogram(&key("histogram"), &identity(), &previous.2)
            .unwrap();
        store
            .save_histogram(&key("histogram"), &identity(), &histogram)
            .unwrap();
    }
    assert!(aggregate.samples_overflowed());
    assert!(aggregate.total_overflowed());
    assert!(histogram.bucket_overflowed()[1]);
}

#[test]
fn identical_checkpoints_validate_without_stable_writes() {
    fn check<T: crate::metric::StoredMetric>(initial: &T, changed: &T) {
        let memory = WriteCountingMemory::default();
        let writes = memory.writes.clone();
        let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
        let key = key("app");
        writes.set(0);
        store.save(&key, &identity(), initial).unwrap();
        assert!(writes.get() > 0); // Absent is different from a retained empty row.
        for (value, changed) in [(initial, false), (changed, true)] {
            writes.set(0);
            store.save(&key, &identity(), value).unwrap();
            assert_eq!(writes.get() > 0, changed);
            writes.set(0);
            store.save(&key, &identity(), value).unwrap();
            assert_eq!(writes.get(), 0);
            assert!(
                store
                    .load::<T>(&key, &identity(), value.configuration())
                    .unwrap()
                    .as_ref()
                    == Some(value)
            );
        }
        writes.set(0);
        assert_eq!(
            store.save(&key, &MetricIdentity::new("bytes", 0).unwrap(), changed),
            Err(StoreError::IdentityMismatch)
        );
        assert_eq!(writes.get(), 0);
        store.map.insert(key.0.clone(), vec![0]);
        writes.set(0);
        assert_eq!(
            store.save(&key, &identity(), changed),
            Err(StoreError::MalformedRecord)
        );
        assert_eq!(writes.get(), 0);
        assert_eq!(store.map.get(&key.0), Some(vec![0]));
    }
    let mut aggregate = Aggregate::EMPTY;
    aggregate.record(0);
    check(&Aggregate::EMPTY, &aggregate);
    let mut counter = Counter::ZERO;
    counter.add(u128::from(u64::MAX) + 1);
    check(&Counter::ZERO, &counter);
    let exact_cap = Counter::from_parts(u128::MAX, false).unwrap();
    let overflowed = Counter::from_parts(u128::MAX, true).unwrap();
    check(&exact_cap, &overflowed);
    let mut summary = Summary::EMPTY;
    summary.record(1);
    check(&Summary::EMPTY, &summary);
    let mut histogram = Histogram::new([0; 0]).unwrap();
    histogram.record(0);
    check(&Histogram::new([]).unwrap(), &histogram);
    let bounds = std::array::from_fn::<_, 256, _>(|index| u64::try_from(index).unwrap());
    let mut histogram = Histogram::new(bounds).unwrap();
    histogram.record(255);
    check(&Histogram::new(bounds).unwrap(), &histogram);
}

#[test]
fn histogram_checkpoints_preserve_each_bucket_count_and_flag() {
    let exact = Aggregate::from_parts(2, 1, Some(1), false, false).unwrap();
    let redistributed = Aggregate::from_parts(3, 3, Some(1), false, false).unwrap();
    let zero = Aggregate::from_parts(u64::MAX, 1, Some(1), true, false).unwrap();
    let total = u128::from(u64::MAX) + 12;
    let overflow = Aggregate::from_parts(u64::MAX, total, Some(11), true, false).unwrap();
    let later_overflow =
        Aggregate::from_parts(u64::MAX, total + 11, Some(11), true, false).unwrap();
    // Every candidate passes core restoration. Other increasing fields cannot
    // compensate for a bucket decrease or a cleared flag in either array slot.
    let pairs = [
        (
            Histogram::from_parts([0, 10], [1, 1], 0, [false; 2], false, exact).unwrap(),
            Histogram::from_parts([0, 10], [0, 3], 0, [false; 2], false, redistributed).unwrap(),
        ),
        (
            Histogram::from_parts([0, 10], [u64::MAX, 1], 0, [true, false], false, zero).unwrap(),
            Histogram::from_parts([0, 10], [u64::MAX, 1], 0, [false; 2], false, zero).unwrap(),
        ),
        (
            Histogram::from_parts([0, 10], [0, u64::MAX], 1, [false, true], false, overflow)
                .unwrap(),
            Histogram::from_parts([0, 10], [0, u64::MAX], 2, [false; 2], false, later_overflow)
                .unwrap(),
        ),
    ];
    for (previous, regressing) in pairs {
        let memory = WriteCountingMemory::default();
        let writes = memory.writes.clone();
        let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
        let key = key("app");
        store.save(&key, &identity(), &previous).unwrap();
        writes.set(0);
        assert_eq!(
            store.save(&key, &identity(), &regressing),
            Err(StoreError::StateRegression)
        );
        assert_eq!(writes.get(), 0);
        assert_eq!(
            store.load_histogram(&key, &identity(), previous.upper_bounds()),
            Ok(Some(previous))
        );
    }
}

#[test]
fn wire_encoding_admits_bounded_histograms_and_numeric_extremes() {
    fn check<T: crate::metric::StoredMetric>(value: &T) {
        let identity = MetricIdentity::new(&"x".repeat(64), u64::MAX).unwrap();
        let bytes = encode(&identity, value);
        assert!(bytes.len() <= codec::MAX_RECORD_BYTES);
        assert!(codec::decode::<T>(&bytes, &identity, value.configuration()).unwrap() == *value);
    }
    check(&Aggregate::EMPTY);
    check(&Counter::ZERO);
    check(&Counter::from_parts(u128::MAX, true).unwrap());
    check(&Summary::EMPTY);
    check(&Histogram::new([]).unwrap());
    let bounds = std::array::from_fn::<_, 256, _>(|i| u64::MAX - 255 + i as u64);
    check(&Histogram::new(bounds).unwrap());
    let wide = Aggregate::from_parts(u64::MAX, u128::MAX, Some(u64::MAX), true, true).unwrap();
    check(&wide);
    check(&Summary::from_parts(wide, Some(u64::MAX)).unwrap());
    check(
        &Histogram::from_parts(
            std::array::from_fn::<_, 256, _>(|i| i as u64),
            [u64::MAX; 256],
            u64::MAX,
            [true; 256],
            true,
            wide,
        )
        .unwrap(),
    );
    assert_eq!(
        <Histogram<257> as crate::metric::StoredMetric>::ADMISSION,
        Err(StoreError::TooManyBounds)
    );
    assert_eq!(
        <Histogram<{ usize::MAX }> as crate::metric::StoredMetric>::ADMISSION,
        Err(StoreError::TooManyBounds)
    );
}

#[test]
fn histogram_equality_shortcut_preserves_full_state_comparison() {
    use crate::metric::StoredMetric;
    fn check<const N: usize>() {
        let bounds = std::array::from_fn::<_, N, _>(|i| i as u64);
        let empty = Histogram::new(bounds).unwrap();
        assert!(empty.same_state(&empty));
        let mut previous = empty;
        previous.record(0);
        assert!(previous.same_state(&previous));
        for sample in [0, u64::MAX] {
            let mut current = previous;
            current.record(sample);
            assert!(!current.same_state(&previous));
        }
        let other_bounds =
            Histogram::new(std::array::from_fn::<_, N, _>(|i| 1 + i as u64)).unwrap();
        assert_eq!(empty.same_state(&other_bounds), N == 0);
        // A clipped aggregate can remain unchanged while distribution changes.
        let aggregate =
            Aggregate::from_parts(u64::MAX, u128::MAX, Some(u64::MAX), true, true).unwrap();
        let clipped =
            Histogram::from_parts(bounds, [0; N], u64::MAX, [false; N], true, aggregate).unwrap();
        let mut continued = clipped;
        continued.record(0);
        assert_eq!(continued.aggregate(), clipped.aggregate());
        assert_eq!(continued.same_state(&clipped), N == 0);
        if N != 0 {
            let mut counts = [0; N];
            counts[0] = u64::MAX;
            let mut flags = [false; N];
            let exact_bucket =
                Histogram::from_parts(bounds, counts, u64::MAX, flags, true, aggregate).unwrap();
            flags[0] = true;
            let overflowed_bucket =
                Histogram::from_parts(bounds, counts, u64::MAX, flags, true, aggregate).unwrap();
            assert!(!exact_bucket.same_state(&overflowed_bucket));
        }
    }
    check::<0>();
    check::<1>();
    check::<4>();
    check::<8>();
    check::<256>();
}

#[test]
fn public_histogram_load_admits_expected_bounds_before_reading_a_row() {
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
    let key = key("app");
    let identity = identity();
    for retained in [None, Some(vec![0])] {
        if let Some(bytes) = retained.as_ref() {
            store.map.insert(key.0.clone(), bytes.clone());
        }
        writes.set(0);
        for bounds in [[10, 10], [10, 0]] {
            assert_eq!(
                store.load_histogram(&key, &identity, &bounds),
                Err(StoreError::InvalidState(
                    ic_metrics::MeasurementStateError::InvalidBounds
                ))
            );
        }
        assert_eq!(
            store.load_histogram(&key, &identity, &[0; 257]),
            Err(StoreError::TooManyBounds)
        );
        assert_eq!(store.map.get(&key.0), retained);
        assert_eq!(writes.get(), 0);
    }
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

#[test]
fn wire_byte_mutations_preserve_canonical_admission_and_rejected_checkpoints() {
    let mut aggregate = Aggregate::EMPTY;
    let mut summary = Summary::EMPTY;
    let mut histogram = Histogram::new([0, 10, u64::MAX]).unwrap();
    for sample in [0, 5, 11] {
        aggregate.record(sample);
        summary.record(sample);
        histogram.record(sample);
    }
    let overflowed =
        Aggregate::from_parts(u64::MAX, u128::MAX, Some(u64::MAX), true, true).unwrap();
    let outcomes = [
        check_wire_mutations(&Counter::ZERO),
        check_wire_mutations(&Counter::from_parts(u128::MAX, false).unwrap()),
        check_wire_mutations(&Counter::from_parts(u128::MAX, true).unwrap()),
        check_wire_mutations(&Aggregate::EMPTY),
        check_wire_mutations(&aggregate),
        check_wire_mutations(&overflowed),
        check_wire_mutations(&Summary::EMPTY),
        check_wire_mutations(&summary),
        check_wire_mutations(&Summary::from_parts(overflowed, Some(u64::MAX)).unwrap()),
        check_wire_mutations(&Histogram::new([]).unwrap()),
        check_wire_mutations(&histogram),
    ];
    assert!(outcomes.iter().all(|(_, refused)| *refused > 0));
    assert!(outcomes.iter().any(|(admitted, _)| *admitted > 0));
}

#[test]
fn checkpoint_continuity_matches_independent_reachable_states() {
    use crate::metric::StoredMetric;

    // Complete alphabet 0..=3: no omitted intermediate observation can explain
    // a transition whose maximum is at most 3. Compare states, not histories.
    let mut sequences = vec![Vec::<u64>::new()];
    for length in 1..=3 {
        for mut code in 0..4_usize.pow(length) {
            let mut sequence = Vec::new();
            for _ in 0..length {
                sequence.push((code % 4) as u64);
                code /= 4;
            }
            sequences.push(sequence);
        }
    }
    let project = |sequence: &[u64]| {
        let mut summary = Summary::EMPTY;
        let mut histogram = Histogram::new([0, 2]).unwrap();
        for &sample in sequence {
            summary.record(sample);
            histogram.record(sample);
        }
        (summary.aggregate(), summary, histogram)
    };
    for prefix in &sequences {
        let previous = project(prefix);
        let reachable: Vec<_> = sequences
            .iter()
            .filter(|suffix| prefix.len() + suffix.len() <= 3)
            .map(|suffix| {
                let mut joined = prefix.clone();
                joined.extend_from_slice(suffix);
                project(&joined)
            })
            .collect();
        for candidate in &sequences {
            let current = project(candidate);
            assert_eq!(
                current.0.continues(&previous.0),
                reachable.iter().any(|state| state.0 == current.0),
                "aggregate {prefix:?} -> {candidate:?}"
            );
            assert_eq!(
                current.1.continues(&previous.1),
                reachable.iter().any(|state| state.1 == current.1),
                "summary {prefix:?} -> {candidate:?}"
            );
            assert_eq!(
                current.2.continues(&previous.2),
                reachable.iter().any(|state| state.2 == current.2),
                "histogram {prefix:?} -> {candidate:?}"
            );
        }
    }
}

#[test]
fn populated_wire_rows_reopen_without_reencoding() {
    // Independent field layout: deliberately do not use the production encoder.
    let id = identity();
    let backing = VectorMemory::default();
    let mut map = StableBTreeMap::<Vec<u8>, Vec<u8>, _>::new(admitted_memory(backing.clone()));
    let maximum = u64::MAX;
    let total = u128::from(maximum) * 2;
    let aggregate = (2_u64, total, Some(maximum), false, false);
    let mut originals = Vec::new();
    for (name, bytes) in [
        ("aggregate", wire_state(1, &id, &aggregate)),
        ("summary", wire_state(2, &id, &(aggregate, Some(maximum)))),
        (
            "histogram",
            wire_state(
                3,
                &id,
                &(aggregate, vec![(maximum, 2_u64, false)], 0_u64, false),
            ),
        ),
        ("counter", wire_state(4, &id, &(u128::MAX, true))),
    ] {
        let key = key(name);
        map.insert(key.0.clone(), bytes.clone());
        originals.push((key, bytes));
    }
    drop(map);
    let store = MetricsStore::new(StableBTreeMap::load(admitted_memory(backing)));
    let expected = Aggregate::from_parts(2, total, Some(maximum), false, false).unwrap();
    assert_eq!(
        store.load_aggregate(&key("aggregate"), &id),
        Ok(Some(expected))
    );
    assert_eq!(
        store.load_summary(&key("summary"), &id),
        Ok(Some(Summary::from_parts(expected, Some(maximum)).unwrap()))
    );
    assert_eq!(
        store.load_histogram(&key("histogram"), &id, &[maximum]),
        Ok(Some(
            Histogram::from_parts([maximum], [2], 0, [false], false, expected).unwrap()
        ))
    );
    assert_eq!(
        store.load_counter(&key("counter"), &id),
        Ok(Some(Counter::from_parts(u128::MAX, true).unwrap()))
    );
    let map = store.into_map();
    for (key, bytes) in originals {
        assert_eq!(map.get(&key.0), Some(bytes));
    }
}

#[test]
fn wire_collection_and_frame_limits_refuse_without_replacement() {
    let memory = WriteCountingMemory::default();
    let writes = memory.writes.clone();
    let mut store = MetricsStore::new(StableBTreeMap::new(admitted_memory(memory)));
    let key = key("limits");
    let identity = identity();
    let value = Histogram::new([]).unwrap();
    let state = (
        (0_u64, 0_u128, None::<u64>, false, false),
        vec![(0_u64, 0_u64, false); 257],
        0_u64,
        false,
    );
    let too_many = wire_state(3, &identity, &state);
    let too_large = vec![0; codec::MAX_RECORD_BYTES + 1];
    for (bytes, error) in [
        (too_many, StoreError::MalformedRecord),
        (too_large, StoreError::RecordTooLarge),
    ] {
        store.map.insert(key.0.clone(), bytes.clone());
        assert_rejected_record(&mut store, &writes, &key, &identity, &value, (bytes, error));
    }
    // The largest syntactic field widths must pass codec budgets even if their
    // impossible distribution is subsequently refused by arithmetic admission.
    let maximum = u64::MAX;
    let state = (
        (maximum, u128::MAX, Some(maximum), true, true),
        (0..256_u64)
            .map(|i| (maximum - 255 + i, maximum, true))
            .collect::<Vec<_>>(),
        maximum,
        true,
    );
    let bytes = wire_state(3, &identity, &state);
    let bounds = std::array::from_fn::<_, 256, _>(|i| maximum - 255 + i as u64);
    assert!(matches!(
        codec::decode::<Histogram<256>>(&bytes, &identity, &bounds),
        Err(StoreError::InvalidState(_))
    ));
}
