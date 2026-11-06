use super::*;
use std::{
    mem::MaybeUninit,
    panic::{AssertUnwindSafe, catch_unwind},
};

struct ReadOnly;
impl Memory for ReadOnly {
    fn size(&self) -> u64 {
        1
    }
    fn grow(&self, _: u64) -> i64 {
        -1
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        assert!(offset + bytes.len() as u64 <= PAGE);
        bytes.fill(37);
    }
    fn write(&self, _: u64, _: &[u8]) {
        panic!("read-only fixture")
    }
}

#[expect(
    unsafe_code,
    reason = "Test-owned independent buffers satisfy the raw-read contract"
)]
fn read<'a>(memory: &impl Memory, offset: u64, bytes: &'a mut [MaybeUninit<u8>]) -> &'a mut [u8] {
    // SAFETY: the supplied buffer is exclusively borrowed and separate from
    // these fixture backings; successful reads initialize its complete extent.
    unsafe {
        memory.read_unsafe(offset, bytes.as_mut_ptr().cast(), bytes.len());
        std::slice::from_raw_parts_mut(bytes.as_mut_ptr().cast(), bytes.len())
    }
}

#[test]
fn default_read_initializes_existing_backings_and_forwards_wrappers() {
    let memory = Rc::new(ReadOnly);
    let mut bytes = [MaybeUninit::uninit(); 17];
    assert_eq!(read(&&memory, 8, &mut bytes), &[37; 17]);
    assert_eq!(read(&memory, PAGE, &mut []), [0_u8; 0]);
    assert!(catch_unwind(AssertUnwindSafe(|| read(&memory, PAGE, &mut bytes).len())).is_err());
}

#[test]
fn native_read_initializes_exact_range_and_rejects_overflow() {
    let memory = VectorMemory::default();
    assert_eq!(memory.grow(1), 0);
    memory.write(PAGE - 4, &[1, 2, 3, 4]);
    let mut bytes = [MaybeUninit::uninit(); 4];
    assert_eq!(read(&memory, PAGE - 4, &mut bytes), &[1, 2, 3, 4]);
    for offset in [PAGE - 3, u64::MAX] {
        assert!(
            catch_unwind(AssertUnwindSafe(|| read(&memory, offset, &mut bytes).len())).is_err()
        );
    }
}

struct DirectRead(VectorMemory);
impl Memory for DirectRead {
    fn size(&self) -> u64 {
        self.0.size()
    }
    fn grow(&self, _: u64) -> i64 {
        -1
    }
    fn read(&self, _: u64, _: &mut [u8]) {
        panic!("raw override must be used")
    }
    fn write(&self, _: u64, _: &[u8]) {
        panic!("read-only fixture")
    }
    #[expect(
        unsafe_code,
        reason = "Forward the unchanged raw destination to a qualified fixture backing"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // SAFETY: the fixture caller guarantees the same exclusive destination.
        unsafe { self.0.read_unsafe(offset, dst, count) }
    }
}

#[test]
fn raw_overrides_survive_shared_and_borrowed_wrappers() {
    let memory = VectorMemory::default();
    memory.grow(1);
    memory.write(3, b"direct");
    let memory = Rc::new(DirectRead(memory));
    let mut bytes = [MaybeUninit::uninit(); 6];
    assert_eq!(read(&&memory, 3, &mut bytes), b"direct");
}
