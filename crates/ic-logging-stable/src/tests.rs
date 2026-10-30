use std::{cell::Cell, num::NonZeroUsize, rc::Rc};

use ic_logging::{AppendError, EntryInput, Filter, Level, PageRequest};
use ic_memory::{
    MemoryBacking, MemoryManagerConfig, RuntimeOpenError, VectorMemory, testing::fixture_runtime,
};

use super::*;
use ic_logging::LogSnapshot;

fn limits() -> Limits {
    Limits {
        max_entries: NonZeroUsize::new(4).unwrap(),
        max_message_bytes: 64,
        max_label_bytes: 16,
        max_text_bytes: MAX_TEXT_BYTES,
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

mod continuity;
mod encoding;
mod storage;
