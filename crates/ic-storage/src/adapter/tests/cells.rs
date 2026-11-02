//! Non-initializing cell admission, canonical bytes and exclusive lease custody.
#![expect(
    clippy::manual_assert_eq,
    reason = "failed byte checks must not print multi-megabyte backing buffers"
)]
use super::*;
use crate::StableCellPayloadError;
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
    refuse_growth: Rc<Cell<bool>>,
}
impl crate::MemoryBacking for ObservedMemory {
    fn backing_identity(&self) -> crate::BackingIdentity {
        self.backing.backing_identity()
    }
}

impl Memory for ObservedMemory {
    fn size(&self) -> u64 {
        self.backing.size()
    }

    fn grow(&self, pages: u64) -> i64 {
        self.grows.set(self.grows.get() + 1);
        if self.refuse_growth.get() {
            return -1;
        }
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
        StorageConfig::default(),
    )
    .unwrap()
}

#[test]
fn load_rejects_empty_foreign_unsupported_and_truncated_cells_without_effects() {
    let mut foreign = [0; crate::STABLE_CELL_HEADER_SIZE];
    foreign[..8].copy_from_slice(b"FOREIGN!");
    let mut invalid = [0; crate::STABLE_CELL_HEADER_SIZE];
    invalid[..8].copy_from_slice(crate::stable_cell::STABLE_CELL_MAGIC);
    invalid[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    for (header, expected) in [
        (None, StableCellPayloadError::NotStableCell),
        (Some(foreign), StableCellPayloadError::NotStableCell),
        (
            Some(invalid),
            StableCellPayloadError::InvalidLength {
                value_len: u64::from(u32::MAX),
                available_bytes: 65_536 - crate::STABLE_CELL_HEADER_SIZE as u64,
            },
        ),
    ] {
        let backing = ObservedMemory::default();
        let runtime = observed_runtime(&backing);
        if let Some(header) = header {
            let memory = runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap();
            memory.grow(1).unwrap();
            memory.write(0, &header);
        }
        let before = backing.snapshot();
        let error = StableCell::<u64, _>::load(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            16 * 1024 * 1024,
        )
        .err()
        .unwrap();
        assert_eq!(error, CollectionError::Cell(expected));
        backing.assert_unchanged(&before);
        assert!(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .is_ok(),
            "failed load retained lease"
        );
    }
}

#[test]
fn codec_errors_and_backing_panics_preserve_memory_and_release_the_consumed_lease() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    // A structurally valid envelope whose single byte cannot decode as u64.
    drop(
        StableCell::create(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            vec![7_u8],
        )
        .unwrap(),
    );
    let before = backing.snapshot();
    assert!(matches!(
        StableCell::<u64, _>::load(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            8
        ),
        Err(CollectionError::Codec(CodecError::Bounds))
    ));
    backing.assert_unchanged(&before);
    assert!(
        runtime
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
            .is_ok()
    );

    let lease = runtime
        .open_memory(&crate::StableKey::parse(KEY).unwrap())
        .unwrap();
    backing.fail_read.set(true);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = StableCell::<Vec<u8>, _>::load(lease, 16 * 1024 * 1024).unwrap();
        }))
        .is_err()
    );
    backing.fail_read.set(false);
    backing.assert_unchanged(&before);
    assert!(matches!(
        runtime.open_memory(&crate::StableKey::parse(KEY).unwrap()),
        Err(RuntimeOpenError::Directory(crate::DirectoryError::Poisoned))
    ));
    drop(runtime);
    let reopened = observed_runtime(&backing);
    assert!(
        reopened
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
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
        fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
            ENCODES.fetch_add(1, Ordering::Relaxed);
            Ok(Cow::Borrowed(&self.0))
        }
        fn into_bytes(self) -> Result<Vec<u8>, CodecError> {
            Ok(self.0)
        }
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
            DECODES.fetch_add(1, Ordering::Relaxed);
            Ok(Self(bytes.into_owned()))
        }
    }
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    let value = vec![42; 70_000];
    drop(
        StableCell::create(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            Record(value.clone()),
        )
        .unwrap(),
    );
    DECODES.store(0, Ordering::Relaxed);
    ENCODES.store(0, Ordering::Relaxed);
    let before = backing.snapshot();
    let mut cell = StableCell::<Record, _>::load(
        runtime
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
            .unwrap(),
        16 * 1024 * 1024,
    )
    .unwrap();
    assert!(cell.get().0 == value);
    assert_eq!(DECODES.load(Ordering::Relaxed), 1);
    assert_eq!(ENCODES.load(Ordering::Relaxed), 0);
    backing.assert_unchanged(&before);
    assert!(matches!(
        runtime.open_memory(&crate::StableKey::parse(KEY).unwrap()),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    assert!(cell.set(Record(Vec::new())).unwrap().0 == value);
    drop(cell);
    let before = backing.snapshot();
    let cell = StableCell::<Record, _>::load(
        runtime
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
            .unwrap(),
        16 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(cell.get().0, [] as [u8; 0]);
    backing.assert_unchanged(&before);
    drop(cell);
    let cell = StableCell::<Record, _>::load(
        runtime
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
            .unwrap(),
        16 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(cell.get().0, [] as [u8; 0]);
}

#[test]
fn create_load_and_explicit_reset_preserve_the_cell_format() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    drop(
        StableCell::create(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            11_u64,
        )
        .unwrap(),
    );
    let before = backing.snapshot();
    let cell = StableCell::<u64, _>::load(
        runtime
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
            .unwrap(),
        16 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(*cell.get(), 11);
    backing.assert_unchanged(&before);
    drop(cell);
    // Explicit reset replaces the value; load preserves the new bytes.
    drop(
        StableCell::reset(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            22_u64,
        )
        .unwrap(),
    );
    let mut cell = StableCell::<u64, _>::load(
        runtime
            .open_memory(&crate::StableKey::parse(KEY).unwrap())
            .unwrap(),
        16 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(*cell.get(), 22);
    assert_eq!(cell.set(33).unwrap(), 22);
    drop(cell);
    let memory = runtime
        .open_memory(&crate::StableKey::parse(KEY).unwrap())
        .unwrap();
    let mut bytes = [0; crate::STABLE_CELL_HEADER_SIZE + 8];
    memory.read(0, &mut bytes);
    assert_eq!(&bytes[..8], crate::stable_cell::STABLE_CELL_MAGIC);
    assert_eq!(&bytes[8..12], &8_u32.to_le_bytes());
    assert_eq!(
        &bytes[crate::STABLE_CELL_HEADER_SIZE..],
        &33_u64.to_le_bytes()
    );
    drop(memory);
    assert_eq!(
        *StableCell::<u64, _>::load(
            runtime
                .open_memory(&crate::StableKey::parse(KEY).unwrap())
                .unwrap(),
            16 * 1024 * 1024
        )
        .unwrap()
        .get(),
        33
    );
}

#[test]
fn cell_page_boundary_growth_refusal_preserves_value_and_allows_retry() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    let key = crate::StableKey::parse(KEY).unwrap();
    let mut cell = StableCell::create(runtime.open_memory(&key).unwrap(), vec![1_u8]).unwrap();

    // Exactly fill one page, including the envelope, without growing.
    let grows = backing.grows.get();
    let full = vec![2; 65_536 - crate::STABLE_CELL_HEADER_SIZE];
    assert_eq!(cell.set(full.clone()).unwrap(), vec![1]);
    assert_eq!(backing.grows.get(), grows);
    let before = backing.snapshot();
    backing.refuse_growth.set(true);
    let larger = vec![3; full.len() + 1];
    assert!(cell.set(larger.clone()).is_err());
    assert!(*backing.backing.borrow() == before.0);
    assert_eq!(backing.writes.get(), before.1);
    assert_eq!(cell.get(), &full);
    assert!(matches!(
        runtime.open_memory(&key),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));

    // A cached read needs neither the backing nor the codec.
    backing.fail_read.set(true);
    assert_eq!(cell.get(), &full);
    backing.fail_read.set(false);
    backing.refuse_growth.set(false);
    assert_eq!(cell.set(larger.clone()).unwrap(), full);
    drop(cell);
    let lease = runtime.open_memory(&key).unwrap();
    assert_eq!(lease.size(), 2);
    let cell = StableCell::<Vec<u8>, _>::load(lease, 16 * 1024 * 1024).unwrap();
    assert_eq!(cell.get(), &larger);
}

fn refuse_collection(
    kind: u8,
    operation: &str,
    runtime: &MemoryRuntime<ObservedMemory>,
) -> Result<(), CollectionError> {
    let memory = runtime
        .open_memory(&crate::StableKey::parse(KEY).unwrap())
        .unwrap();
    let create = operation == "create";
    match kind {
        0 => {
            if create {
                StableCell::create(memory, 7_u64).map(drop)
            } else {
                StableCell::<u64, _>::load(memory, 8).map(drop)
            }
        }
        1 => {
            if create {
                StableBTreeMap::<u64, u64, _>::create(memory).map(drop)
            } else {
                StableBTreeMap::<u64, u64, _>::load(memory).map(drop)
            }
        }
        2 => {
            if create {
                StableBTreeSet::<u64, _>::create(memory).map(drop)
            } else {
                StableBTreeSet::<u64, _>::load(memory).map(drop)
            }
        }
        3 => {
            if create {
                StableVec::<u64, _>::create(memory).map(drop)
            } else {
                StableVec::<u64, _>::load(memory).map(drop)
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn lifecycle_refusals_preserve_bytes_writes_growth_and_release_every_lease() {
    for kind in 0..4 {
        for operation in ["create", "load-empty", "load-foreign"] {
            let backing = ObservedMemory::default();
            let runtime = observed_runtime(&backing);
            if operation != "load-empty" {
                let memory = runtime
                    .open_memory(&crate::StableKey::parse(KEY).unwrap())
                    .unwrap();
                memory.grow(1).unwrap();
                memory.write(0, b"BAD\0\0\0\0\0");
            }
            let before = backing.snapshot();
            assert!(refuse_collection(kind, operation, &runtime).is_err());
            backing.assert_unchanged(&before);
            assert!(
                runtime
                    .open_memory(&crate::StableKey::parse(KEY).unwrap())
                    .is_ok()
            );
        }
    }
}

#[test]
fn unbounded_cells_keep_explicit_lifecycle_contracts() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    let key = crate::StableKey::parse(KEY).unwrap();
    let cell = StableCell::create(runtime.open_memory(&key).unwrap(), vec![3_u8; 17]).unwrap();
    drop(cell);
    let cell = StableCell::<Vec<u8>, _>::load(runtime.open_memory(&key).unwrap(), 16 * 1024 * 1024)
        .unwrap();
    assert_eq!(cell.get(), &vec![3; 17]);
    drop(cell);
}
