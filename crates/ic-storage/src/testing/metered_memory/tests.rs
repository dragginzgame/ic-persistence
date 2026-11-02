use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn clones_share_bytes_counts_and_reset_without_counting_observation() {
    let memory = MeteredMemory::<VectorMemory>::default();
    let alias = memory.clone();
    assert_eq!(memory.grow(1), 0);
    memory.write(0, b"abc");
    let mut bytes = [0; 3];
    alias.read(0, &mut bytes);
    assert_eq!(&bytes, b"abc");
    assert_eq!(alias.backing().size(), 1);
    assert_eq!(
        alias.counts(),
        MemoryIoCounts {
            reads: 1,
            read_bytes: 3,
            writes: 1,
            write_bytes: 3,
            grows: 1,
            grown_pages: 1,
            overflowed: false,
        }
    );
    alias.reset();
    assert_eq!(memory.counts(), MemoryIoCounts::default());
    memory.read(0, &mut bytes);
    assert_eq!(&bytes, b"abc");
}

struct Refusing;
impl Memory for Refusing {
    fn size(&self) -> u64 {
        1
    }
    fn grow(&self, pages: u64) -> i64 {
        if pages == 0 { 1 } else { -1 }
    }
    fn read(&self, _: u64, _: &mut [u8]) {
        panic!("injected read failure")
    }
    fn write(&self, _: u64, _: &[u8]) {
        panic!("injected write failure")
    }
}

#[test]
fn refused_growth_and_panicking_io_preserve_backing_behavior_and_attempts() {
    let memory = MeteredMemory::new(Refusing);
    assert_eq!(memory.grow(0), 1);
    assert_eq!(memory.grow(9), -1);
    assert!(catch_unwind(AssertUnwindSafe(|| memory.read(0, &mut [0; 3]))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| memory.write(0, b"abc"))).is_err());
    assert_eq!(
        memory.counts(),
        MemoryIoCounts {
            reads: 1,
            read_bytes: 3,
            writes: 1,
            write_bytes: 3,
            grows: 2,
            ..MemoryIoCounts::default()
        }
    );
}

#[test]
fn actual_counter_overflow_is_sticky_until_shared_reset() {
    for (index, counts) in [
        MemoryIoCounts {
            reads: u64::MAX,
            ..MemoryIoCounts::default()
        },
        MemoryIoCounts {
            read_bytes: u64::MAX,
            ..MemoryIoCounts::default()
        },
        MemoryIoCounts {
            writes: u64::MAX,
            ..MemoryIoCounts::default()
        },
        MemoryIoCounts {
            write_bytes: u64::MAX,
            ..MemoryIoCounts::default()
        },
        MemoryIoCounts {
            grows: u64::MAX,
            ..MemoryIoCounts::default()
        },
        MemoryIoCounts {
            grown_pages: u64::MAX,
            ..MemoryIoCounts::default()
        },
    ]
    .into_iter()
    .enumerate()
    {
        let memory = MeteredMemory::<VectorMemory>::default();
        memory.grow(1);
        memory.write(0, b"X");
        memory.state.counts.set(counts);
        let alias = memory.clone();
        let mut byte = [0];
        match index {
            0 | 1 => memory.read(0, &mut byte),
            2 | 3 => memory.write(0, b"X"),
            4 => {
                memory.grow(0);
            }
            5 => {
                memory.grow(1);
            }
            _ => unreachable!(),
        }
        let counts = alias.counts();
        assert!(counts.overflowed, "counter {index}");
        assert_eq!(
            [
                counts.reads,
                counts.read_bytes,
                counts.writes,
                counts.write_bytes,
                counts.grows,
                counts.grown_pages
            ][index],
            u64::MAX
        );
        memory.read(0, &mut []);
        assert!(alias.counts().overflowed);
        alias.reset();
        assert_eq!(memory.counts(), MemoryIoCounts::default());
        memory.backing().read(0, &mut byte);
        assert_eq!(byte, *b"X");
        assert_eq!(memory.counts(), MemoryIoCounts::default());
    }
}

#[test]
fn zero_deltas_at_counter_limit_do_not_signal_overflow() {
    let memory = MeteredMemory::<VectorMemory>::default();
    memory.grow(1);
    memory.state.counts.set(MemoryIoCounts {
        read_bytes: u64::MAX,
        write_bytes: u64::MAX,
        grown_pages: u64::MAX,
        ..MemoryIoCounts::default()
    });
    memory.read(0, &mut []);
    memory.write(0, &[]);
    assert_eq!(memory.grow(0), 1);
    assert_eq!(
        memory.counts(),
        MemoryIoCounts {
            reads: 1,
            read_bytes: u64::MAX,
            writes: 1,
            write_bytes: u64::MAX,
            grows: 1,
            grown_pages: u64::MAX,
            overflowed: false,
        }
    );
}

#[test]
fn wrapping_and_cloning_preserve_exclusive_physical_backing_custody() {
    use crate::{MemoryRuntime, RuntimeConstructionError};
    let backing = VectorMemory::default();
    let memory = MeteredMemory::new(backing.clone());
    let alias = memory.clone();
    let independently_wrapped = MeteredMemory::new(backing.clone());
    assert_eq!(memory.backing_identity(), backing.backing_identity());
    assert_eq!(
        alias.backing_identity(),
        independently_wrapped.backing_identity()
    );
    assert_ne!(
        memory.backing_identity(),
        VectorMemory::default().backing_identity()
    );
    assert_eq!(memory.counts(), MemoryIoCounts::default());
    let runtime = MemoryRuntime::new(memory).unwrap();
    assert!(matches!(
        MemoryRuntime::new(alias),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    assert!(matches!(
        MemoryRuntime::new(independently_wrapped),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    assert!(matches!(
        MemoryRuntime::new(backing.clone()),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    drop(runtime);
    assert!(MemoryRuntime::new(backing).is_ok());
}
