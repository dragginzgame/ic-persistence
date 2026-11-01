//! Non-initializing cell admission, canonical bytes and exclusive lease custody.
#![expect(
    clippy::manual_assert_eq,
    reason = "failed byte checks must not print multi-megabyte backing buffers"
)]
use super::*;
use ic_memory::StableCellPayloadError;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::{AtomicUsize, Ordering},
};

const KEY: &str = "test.cell.v1";

#[derive(Clone, Default)]
struct ObservedMemory {
    backing: VectorMemory,
    writes: Rc<Cell<u64>>,
    grows: Rc<Cell<u64>>,
    fail_read: Rc<Cell<bool>>,
}
impl ic_memory::MemoryBacking for ObservedMemory {
    fn backing_identity(&self) -> ic_memory::BackingIdentity {
        self.backing.backing_identity()
    }
}

impl Memory for ObservedMemory {
    fn size(&self) -> u64 {
        self.backing.size()
    }

    fn grow(&self, pages: u64) -> i64 {
        self.grows.set(self.grows.get() + 1);
        self.backing.grow(pages)
    }

    fn read(&self, offset: u64, bytes: &mut [u8]) {
        assert!(!self.fail_read.get(), "injected backing read failure");
        self.backing.read(offset, bytes);
    }

    fn write(&self, offset: u64, bytes: &[u8]) {
        self.writes.set(self.writes.get() + 1);
        self.backing.write(offset, bytes);
    }
}

impl ObservedMemory {
    fn snapshot(&self) -> (Vec<u8>, u64, u64) {
        (
            self.backing.borrow().clone(),
            self.writes.get(),
            self.grows.get(),
        )
    }

    fn assert_unchanged(&self, before: &(Vec<u8>, u64, u64)) {
        assert!(*self.backing.borrow() == before.0, "backing bytes changed");
        assert_eq!(self.writes.get(), before.1, "load attempted a write");
        assert_eq!(self.grows.get(), before.2, "load attempted growth");
    }
}

fn observed_runtime(memory: &ObservedMemory) -> MemoryRuntime<ObservedMemory> {
    fixture_runtime(
        memory.clone(),
        "test",
        "test.",
        &KEYS,
        MemoryManagerConfig::new(std::num::NonZeroU16::new(1).unwrap()),
    )
    .unwrap()
}

#[test]
fn load_rejects_empty_foreign_unsupported_and_truncated_cells_without_effects() {
    for (header, expected) in [
        (None, StableCellPayloadError::NotStableCell),
        (
            Some(*b"BTR\x01\0\0\0\0"),
            StableCellPayloadError::NotStableCell,
        ),
        (
            Some(*b"SCL\x02\0\0\0\0"),
            StableCellPayloadError::UnexpectedLayoutVersion { version: 2 },
        ),
        (
            Some(*b"SCL\x01\xff\xff\xff\xff"),
            StableCellPayloadError::InvalidLength {
                value_len: u64::from(u32::MAX),
                available_bytes: 65_536 - 8,
            },
        ),
    ] {
        let backing = ObservedMemory::default();
        let runtime = observed_runtime(&backing);
        if let Some(header) = header {
            let memory = runtime
                .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
                .unwrap();
            memory.grow(1).unwrap();
            memory.write(0, &header);
        }
        let before = backing.snapshot();
        let error = StableCell::<u64, _>::load(
            runtime
                .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
                .unwrap(),
        )
        .err()
        .unwrap();
        assert_eq!(error, expected);
        backing.assert_unchanged(&before);
        assert!(
            runtime
                .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
                .is_ok(),
            "failed load retained lease"
        );
    }
}

#[test]
fn codec_and_backing_panics_preserve_memory_and_release_the_consumed_lease() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    // A structurally valid envelope whose single byte cannot decode as u64.
    drop(substrate::Cell::new(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
        vec![7_u8],
    ));
    let before = backing.snapshot();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = StableCell::<u64, _>::load(
                runtime
                    .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
                    .unwrap(),
            );
        }))
        .is_err()
    );
    backing.assert_unchanged(&before);
    assert!(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .is_ok()
    );

    let lease = runtime
        .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
        .unwrap();
    backing.fail_read.set(true);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = StableCell::<Vec<u8>, _>::load(lease);
        }))
        .is_err()
    );
    backing.fail_read.set(false);
    backing.assert_unchanged(&before);
    assert!(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .is_ok()
    );
}

#[test]
fn load_decodes_once_without_a_default_or_encoding_and_retains_canonical_bytes() {
    static DECODES: AtomicUsize = AtomicUsize::new(0);
    static ENCODES: AtomicUsize = AtomicUsize::new(0);
    // Deliberately neither Clone nor Default. Values larger than the ledger
    // recovery ceiling are valid collection payloads, as are empty payloads.
    struct Record(Vec<u8>);
    impl Storable for Record {
        const BOUND: Bound = Bound::Unbounded;
        fn to_bytes(&self) -> Cow<'_, [u8]> {
            ENCODES.fetch_add(1, Ordering::Relaxed);
            Cow::Borrowed(&self.0)
        }
        fn into_bytes(self) -> Vec<u8> {
            self.0
        }
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
            DECODES.fetch_add(1, Ordering::Relaxed);
            Self(bytes.into_owned())
        }
    }
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    let value = vec![42; 70_000];
    // Producer is the qualified upstream Cell, not our wrapper.
    drop(substrate::Cell::new(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
        Record(value.clone()),
    ));
    DECODES.store(0, Ordering::Relaxed);
    ENCODES.store(0, Ordering::Relaxed);
    let before = backing.snapshot();
    let mut cell = StableCell::<Record, _>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(cell.get().0 == value);
    assert_eq!(DECODES.load(Ordering::Relaxed), 1);
    assert_eq!(ENCODES.load(Ordering::Relaxed), 0);
    backing.assert_unchanged(&before);
    assert!(matches!(
        runtime.open_memory(&ic_memory::StableKey::parse(KEY).unwrap()),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    assert!(cell.set(Record(Vec::new())).0 == value);
    drop(cell);
    let before = backing.snapshot();
    let cell = StableCell::<Record, _>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(cell.get().0, [] as [u8; 0]);
    backing.assert_unchanged(&before);
    drop(cell);
    // Consumer is upstream too; it recognizes the unchanged envelope/codec.
    let cell = substrate::Cell::init(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
        Record(vec![99]),
    );
    assert_eq!(cell.get().0, [] as [u8; 0]);
}

#[test]
fn create_load_and_explicit_reset_preserve_the_upstream_format() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    drop(StableCell::create(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
        11_u64,
    ));
    let before = backing.snapshot();
    let cell = StableCell::<u64, _>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(*cell.get(), 11);
    backing.assert_unchanged(&before);
    drop(cell);
    // Explicit reset replaces the value; load preserves the new bytes.
    drop(StableCell::reset(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
        22_u64,
    ));
    let mut cell = StableCell::<u64, _>::load(
        runtime
            .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(*cell.get(), 22);
    assert_eq!(cell.set(33), 22);
    drop(cell);
    let memory = runtime
        .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
        .unwrap();
    let mut bytes = [0; 16];
    memory.read(0, &mut bytes);
    assert_eq!(&bytes[..8], b"SCL\x01\x08\0\0\0");
    assert_eq!(&bytes[8..], &33_u64.to_be_bytes());
    drop(memory);
    assert_eq!(
        *StableCell::<u64, _>::load(
            runtime
                .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
                .unwrap()
        )
        .unwrap()
        .get(),
        33
    );
}

fn refuse_collection(kind: u8, operation: &str, runtime: &MemoryRuntime<ObservedMemory>) {
    let memory = runtime
        .open_memory(&ic_memory::StableKey::parse(KEY).unwrap())
        .unwrap();
    let create = operation == "create";
    match kind {
        0 => {
            if create {
                drop(StableCell::create(memory, 7_u64));
            } else {
                drop(StableCell::<u64, _>::load(memory).unwrap());
            }
        }
        1 => {
            drop(if create {
                StableBTreeMap::<u64, u64, _>::create(memory)
            } else {
                StableBTreeMap::load(memory)
            });
        }
        2 => {
            drop(if create {
                StableBTreeSet::<u64, _>::create(memory)
            } else {
                StableBTreeSet::load(memory)
            });
        }
        3 => {
            drop(if create {
                StableVec::<u64, _>::create(memory)
            } else {
                StableVec::load(memory)
            });
        }
        4 => {
            drop(if create {
                StableMinHeap::<u64, _>::create(memory)
            } else {
                StableMinHeap::load(memory)
            });
        }
        _ => {
            let data = runtime
                .open_memory(&ic_memory::StableKey::parse("test.log.data.v1").unwrap())
                .unwrap();
            drop(if create {
                StableLog::<u64, _>::create(memory, data)
            } else {
                StableLog::load(memory, data)
            });
        }
    }
}

#[test]
fn lifecycle_refusals_preserve_bytes_writes_growth_and_release_every_lease() {
    for kind in 0..6 {
        for operation in ["create", "load-empty", "load-foreign"] {
            let backing = ObservedMemory::default();
            let runtime = observed_runtime(&backing);
            if operation != "load-empty" {
                for key in [KEY, "test.log.data.v1"] {
                    let memory = runtime
                        .open_memory(&ic_memory::StableKey::parse(key).unwrap())
                        .unwrap();
                    memory.grow(1).unwrap();
                    memory.write(0, b"BAD\0\0\0\0\0");
                }
            }
            let before = backing.snapshot();
            assert!(
                catch_unwind(AssertUnwindSafe(|| refuse_collection(
                    kind, operation, &runtime
                )))
                .is_err()
            );
            backing.assert_unchanged(&before);
            for key in [KEY, "test.log.data.v1"] {
                assert!(
                    runtime
                        .open_memory(&ic_memory::StableKey::parse(key).unwrap())
                        .is_ok()
                );
            }
        }
    }
}

#[test]
fn unbounded_cells_and_log_index_corruption_keep_explicit_lifecycle_contracts() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    let key = ic_memory::StableKey::parse(KEY).unwrap();
    let cell = StableCell::create(runtime.open_memory(&key).unwrap(), vec![3_u8; 17]);
    drop(cell);
    let cell = StableCell::<Vec<u8>, _>::load(runtime.open_memory(&key).unwrap()).unwrap();
    assert_eq!(cell.get(), &vec![3; 17]);
    drop(cell);
    let data_key = ic_memory::StableKey::parse("test.log.data.v1").unwrap();
    let log = StableLog::<u64, _>::reset(
        runtime.open_memory(&key).unwrap(),
        runtime.open_memory(&data_key).unwrap(),
    );
    log.append(&9).unwrap();
    drop(log);
    let index = runtime.open_memory(&key).unwrap();
    index.write(0, b"BAD");
    drop(index);
    let before = backing.snapshot();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = StableLog::<u64, _>::load(
                runtime.open_memory(&key).unwrap(),
                runtime.open_memory(&data_key).unwrap(),
            );
        }))
        .is_err()
    );
    backing.assert_unchanged(&before);
}

#[test]
fn log_lifecycle_refuses_partially_initialized_pairs_without_effects() {
    for occupied in [0, 1] {
        let backing = ObservedMemory::default();
        let runtime = observed_runtime(&backing);
        let keys = [KEY, "test.log.data.v1"].map(|key| ic_memory::StableKey::parse(key).unwrap());
        let memory = runtime.open_memory(&keys[occupied]).unwrap();
        memory.grow(1).unwrap();
        memory.write(0, b"BAD");
        drop(memory);
        for create in [true, false] {
            let before = backing.snapshot();
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    let index = runtime.open_memory(&keys[0]).unwrap();
                    let data = runtime.open_memory(&keys[1]).unwrap();
                    drop(if create {
                        StableLog::<u64, _>::create(index, data)
                    } else {
                        StableLog::load(index, data)
                    });
                }))
                .is_err()
            );
            backing.assert_unchanged(&before);
            for key in &keys {
                assert!(runtime.open_memory(key).is_ok());
            }
        }
    }
}
