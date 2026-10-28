//! Physical-backing ownership and divergence refusal at the host boundary.
#![expect(
    clippy::manual_assert_eq,
    reason = "do not print entire stable memories on failure"
)]
use super::*;
use crate::{MemoryBacking, VectorMemory, testing::fixture_runtime};
use std::panic::{AssertUnwindSafe, catch_unwind};

const A: &str = "custody.a.v1";
const B: &str = "custody.b.v1";
const C: &str = "custody.c.v1";

#[test]
fn duplicate_construction_calls_no_backing_method() {
    #[derive(Clone)]
    struct NoAccess {
        backing: VectorMemory,
        refuse: Rc<std::cell::Cell<bool>>,
    }
    impl MemoryBacking for NoAccess {
        fn backing_identity(&self) -> BackingIdentity {
            self.backing.backing_identity()
        }
    }
    impl Memory for NoAccess {
        fn size(&self) -> u64 {
            assert!(!self.refuse.get());
            self.backing.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            assert!(!self.refuse.get());
            self.backing.grow(pages)
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            assert!(!self.refuse.get());
            self.backing.read(offset, dst);
        }
        fn write(&self, offset: u64, src: &[u8]) {
            assert!(!self.refuse.get());
            self.backing.write(offset, src);
        }
    }
    let memory = NoAccess {
        backing: VectorMemory::default(),
        refuse: Rc::default(),
    };
    let owner = MemoryRuntime::new(memory.clone()).unwrap();
    memory.refuse.set(true);
    assert!(matches!(
        MemoryRuntime::new(memory.clone()),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    memory.refuse.set(false);
    drop(owner);
    assert!(MemoryRuntime::new(memory).is_ok());
}

fn runtime(backing: VectorMemory) -> MemoryRuntime<VectorMemory> {
    fixture_runtime(
        backing,
        "custody",
        "custody.",
        &[A, B, C],
        MemoryManagerConfig::new(8).unwrap(),
    )
    .unwrap()
}

#[test]
fn aliases_refuse_before_io_and_detached_leases_keep_the_backing_claimed() {
    let backing = VectorMemory::default();
    let owner = runtime(backing.clone());
    let a = owner.open_memory(A).unwrap();
    let b = owner.open_memory(B).unwrap();
    a.grow(1).unwrap();
    b.grow(1).unwrap();
    a.write(17, &[42]);
    b.write(31, &[99]);
    let before = backing.borrow().clone();
    for _ in 0..3 {
        assert!(matches!(
            MemoryRuntime::new(backing.clone()),
            Err(RuntimeConstructionError::BackingAlreadyClaimed)
        ));
        assert!(*backing.borrow() == before);
    }
    drop(owner);
    assert!(matches!(
        MemoryRuntime::new(backing.clone()),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    a.grow(1).unwrap();
    a.write(65_537, &[7]);
    drop(a);
    assert!(matches!(
        MemoryRuntime::new(backing.clone()),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    drop(b);
    let owner = runtime(backing);
    let a = owner.open_memory(A).unwrap();
    let b = owner.open_memory(B).unwrap();
    let mut byte = [0];
    a.read(17, &mut byte);
    assert_eq!(byte, [42]);
    a.read(65_537, &mut byte);
    assert_eq!(byte, [7]);
    b.read(31, &mut byte);
    assert_eq!(byte, [99]);
}

#[test]
fn independent_backings_and_rc_aliases_follow_physical_identity() {
    let a = VectorMemory::default();
    let b = VectorMemory::default();
    let first = runtime(a.clone());
    let second = runtime(b.clone());
    assert!(matches!(
        MemoryRuntime::new(Rc::new(a.clone())),
        Err(RuntimeConstructionError::BackingAlreadyClaimed)
    ));
    drop(first);
    drop(second);
    assert!(MemoryRuntime::new(a).is_ok());
    assert!(MemoryRuntime::new(b).is_ok());
}

#[test]
fn failed_construction_and_unwinding_release_the_claim() {
    struct PanicWrite {
        backing: VectorMemory,
    }
    impl MemoryBacking for PanicWrite {
        fn backing_identity(&self) -> BackingIdentity {
            self.backing.backing_identity()
        }
    }
    impl Memory for PanicWrite {
        fn size(&self) -> u64 {
            self.backing.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            self.backing.grow(pages)
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            self.backing.read(offset, dst);
        }
        fn write(&self, _: u64, _: &[u8]) {
            panic!("injected metadata write failure");
        }
    }
    let backing = VectorMemory::default();
    backing.grow(1);
    assert!(matches!(
        MemoryRuntime::new(backing.clone()),
        Err(RuntimeConstructionError::ForeignMemory { .. })
    ));
    backing.borrow_mut().clear();
    assert!(
        catch_unwind(AssertUnwindSafe(|| MemoryRuntime::new(PanicWrite {
            backing: backing.clone()
        })))
        .is_err()
    );
    backing.borrow_mut().clear();
    assert!(MemoryRuntime::new(backing).is_ok());
}

#[test]
#[expect(
    unsafe_code,
    reason = "exercise refusal with a valid separate raw destination"
)]
fn foreign_manager_changes_poison_at_open_or_growth_before_mutation() {
    for detect_at_open in [false, true] {
        for mode in 0..4 {
            let backing = VectorMemory::default();
            let owner = runtime(backing.clone());
            let a = owner.open_memory(A).unwrap();
            let b = owner.open_memory(B).unwrap();
            a.grow(1).unwrap();
            b.grow(1).unwrap();
            a.write(17, &[42]);
            b.write(31, &[99]);
            let id = owner.memory_id(A).unwrap();
            let saved = backing.borrow().clone();
            // Infrastructure fault injection: never a supported production manager.
            let foreign = MemoryManager::init(backing.clone());
            match mode {
                0 => {
                    assert_eq!(foreign.get(MemoryId::new(240)).grow(1), 0);
                }
                1 => {
                    assert_eq!(foreign.get(MemoryId::new(id)).grow(1), 1);
                }
                2 => {
                    let other = owner.memory_id(B).unwrap();
                    let mut bytes = backing.borrow_mut();
                    let table = &mut bytes[layout::HEADER_BYTES..layout::METADATA_BYTES];
                    let ai = table.iter().position(|value| *value == id).unwrap();
                    let bi = table.iter().position(|value| *value == other).unwrap();
                    table.swap(ai, bi);
                }
                _ => {
                    backing.borrow_mut()[layout::METADATA_BYTES - 1] = 0;
                }
            }
            let before = backing.borrow().clone();
            if detect_at_open {
                assert!(matches!(
                    owner.open_memory(C),
                    Err(RuntimeOpenError::Layout(
                        MemoryManagerLayoutError::RuntimeMismatch
                    ))
                ));
            } else {
                assert_eq!(
                    a.grow(0),
                    Err(RuntimeGrowError::Layout(
                        MemoryManagerLayoutError::RuntimeMismatch
                    ))
                );
            }
            assert_eq!(
                a.grow(1),
                Err(RuntimeGrowError::Layout(
                    MemoryManagerLayoutError::RuntimeMismatch
                ))
            );
            assert!(matches!(
                owner.open_memory(C),
                Err(RuntimeOpenError::Layout(
                    MemoryManagerLayoutError::RuntimeMismatch
                ))
            ));
            assert!(catch_unwind(AssertUnwindSafe(|| a.size())).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| a.read(17, &mut [0]))).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| a.write(17, &[0]))).is_err());
            let mut destination = [0xA5];
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    // SAFETY: The single-byte destination remains valid and disjoint.
                    unsafe {
                        a.read_unsafe(17, destination.as_mut_ptr(), 1);
                    }
                }))
                .is_err()
            );
            assert_eq!(destination, [0xA5]);
            assert!(*backing.borrow() == before, "refusal changed bytes");
            // Restoring the header does not silently rehabilitate a poisoned owner.
            *backing.borrow_mut() = saved;
            assert_eq!(
                a.grow(0),
                Err(RuntimeGrowError::Layout(
                    MemoryManagerLayoutError::RuntimeMismatch
                ))
            );
            drop(a);
            drop(b);
            drop(owner);
            drop(foreign);
            let reopened = runtime(backing);
            let mut byte = [0];
            reopened.open_memory(A).unwrap().read(17, &mut byte);
            assert_eq!(byte, [42]);
            reopened.open_memory(B).unwrap().read(31, &mut byte);
            assert_eq!(byte, [99]);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn shared_native_backing_cannot_be_claimed_from_another_thread() {
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);
    impl MemoryBacking for Shared {
        fn backing_identity(&self) -> BackingIdentity {
            BackingIdentity::of_arc(&self.0)
        }
    }
    impl Memory for Shared {
        fn size(&self) -> u64 {
            (self.0.lock().unwrap().len() / 65_536) as u64
        }
        fn grow(&self, pages: u64) -> i64 {
            let mut bytes = self.0.lock().unwrap();
            let old = bytes.len() / 65_536;
            bytes.resize(old * 65_536 + usize::try_from(pages).unwrap() * 65_536, 0);
            i64::try_from(old).unwrap()
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            let bytes = self.0.lock().unwrap();
            let start = usize::try_from(offset).unwrap();
            dst.copy_from_slice(&bytes[start..start + dst.len()]);
        }
        fn write(&self, offset: u64, src: &[u8]) {
            let mut bytes = self.0.lock().unwrap();
            let start = usize::try_from(offset).unwrap();
            bytes[start..start + src.len()].copy_from_slice(src);
        }
    }
    let memory = Shared::default();
    let owner = MemoryRuntime::new(memory.clone()).unwrap();
    let alias = memory.clone();
    assert_eq!(
        std::thread::spawn(move || MemoryRuntime::new(alias).err())
            .join()
            .unwrap(),
        Some(RuntimeConstructionError::BackingAlreadyClaimed)
    );
    drop(owner);
    assert!(
        std::thread::spawn(move || MemoryRuntime::new(memory).is_ok())
            .join()
            .unwrap()
    );
}
