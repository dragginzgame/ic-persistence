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
        MemoryManagerConfig::new(1).unwrap(),
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
            let memory = runtime.open_memory(KEY).unwrap();
            memory.grow(1).unwrap();
            memory.write(0, &header);
        }
        let before = backing.snapshot();
        let error = StableCell::<u64, _>::load(runtime.open_memory(KEY).unwrap())
            .err()
            .unwrap();
        assert_eq!(error, expected);
        backing.assert_unchanged(&before);
        assert!(
            runtime.open_memory(KEY).is_ok(),
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
        runtime.open_memory(KEY).unwrap(),
        vec![7_u8],
    ));
    let before = backing.snapshot();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = StableCell::<u64, _>::load(runtime.open_memory(KEY).unwrap());
        }))
        .is_err()
    );
    backing.assert_unchanged(&before);
    assert!(runtime.open_memory(KEY).is_ok());

    let lease = runtime.open_memory(KEY).unwrap();
    backing.fail_read.set(true);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _ = StableCell::<Vec<u8>, _>::load(lease);
        }))
        .is_err()
    );
    backing.fail_read.set(false);
    backing.assert_unchanged(&before);
    assert!(runtime.open_memory(KEY).is_ok());
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
        runtime.open_memory(KEY).unwrap(),
        Record(value.clone()),
    ));
    DECODES.store(0, Ordering::Relaxed);
    ENCODES.store(0, Ordering::Relaxed);
    let before = backing.snapshot();
    let mut cell = StableCell::<Record, _>::load(runtime.open_memory(KEY).unwrap()).unwrap();
    assert!(cell.get().0 == value);
    assert_eq!(DECODES.load(Ordering::Relaxed), 1);
    assert_eq!(ENCODES.load(Ordering::Relaxed), 0);
    backing.assert_unchanged(&before);
    assert!(matches!(
        runtime.open_memory(KEY),
        Err(RuntimeOpenError::AlreadyOpen { .. })
    ));
    assert!(cell.set(Record(Vec::new())).0 == value);
    drop(cell);
    let before = backing.snapshot();
    let cell = StableCell::<Record, _>::load(runtime.open_memory(KEY).unwrap()).unwrap();
    assert_eq!(cell.get().0, [] as [u8; 0]);
    backing.assert_unchanged(&before);
    drop(cell);
    // Consumer is upstream too; it recognizes the unchanged envelope/codec.
    let cell = substrate::Cell::init(runtime.open_memory(KEY).unwrap(), Record(vec![99]));
    assert_eq!(cell.get().0, [] as [u8; 0]);
}

#[test]
fn new_init_load_and_set_interoperate_without_changing_initialization_contracts() {
    let backing = ObservedMemory::default();
    let runtime = observed_runtime(&backing);
    drop(StableCell::init(runtime.open_memory(KEY).unwrap(), 11_u64));
    let before = backing.snapshot();
    let cell = StableCell::init(runtime.open_memory(KEY).unwrap(), 99_u64);
    assert_eq!(*cell.get(), 11);
    backing.assert_unchanged(&before);
    drop(cell);
    // new remains destructive; load retains its replacement and set's old value.
    drop(StableCell::new(runtime.open_memory(KEY).unwrap(), 22_u64));
    let mut cell = StableCell::<u64, _>::load(runtime.open_memory(KEY).unwrap()).unwrap();
    assert_eq!(*cell.get(), 22);
    assert_eq!(cell.set(33), 22);
    drop(cell);
    let memory = runtime.open_memory(KEY).unwrap();
    let mut bytes = [0; 16];
    memory.read(0, &mut bytes);
    assert_eq!(&bytes[..8], b"SCL\x01\x08\0\0\0");
    assert_eq!(&bytes[8..], &33_u64.to_be_bytes());
    drop(memory);
    assert_eq!(
        *StableCell::<u64, _>::load(runtime.open_memory(KEY).unwrap())
            .unwrap()
            .get(),
        33
    );
}
