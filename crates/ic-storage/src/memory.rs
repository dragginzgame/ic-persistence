//! Storage-owned backing interface and native/IC implementations.
use crate::WASM_PAGE_SIZE_BYTES as PAGE;
use std::{cell::RefCell, rc::Rc};

///
/// Memory
///
/// Page-addressed backing for Storage. Growth returns the previous page count,
/// or -1 without changing memory. Successful growth adds zero-filled pages.
/// Reads and writes must reject the complete out-of-bounds range before IO.
/// Implementing this trait alone does not grant allocation authority.
///
pub trait Memory {
    /// Current extent in 65,536-byte pages.
    fn size(&self) -> u64;
    /// Add zero-filled pages, or return -1 on refusal.
    fn grow(&self, pages: u64) -> i64;
    /// Read an admitted byte range.
    fn read(&self, offset: u64, bytes: &mut [u8]);
    /// Read into a potentially uninitialized destination, following ICSS's hook.
    ///
    /// Every implementation must initialize all `count` bytes on successful
    /// return. A failed read may leave a partially initialized destination.
    /// The default supports existing backings through [`Self::read`]; overrides
    /// can avoid zeroing bytes that backing IO will overwrite.
    ///
    /// # Safety
    /// The caller must provide an exclusive writable `count`-byte range valid
    /// for a Rust slice. It must not overlap this memory's state or read source.
    #[inline]
    #[expect(
        unsafe_code,
        reason = "ICSS raw-read contract admits an exclusive destination and guarantees initialization"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // Adapted from ICSS 0.7.2 Memory::read_unsafe (Apache-2.0).
        // SAFETY: the caller admits the range; zeroing makes the u8 slice valid.
        unsafe {
            std::ptr::write_bytes(dst, 0, count);
            self.read(offset, std::slice::from_raw_parts_mut(dst, count));
        }
    }
    /// Write an admitted byte range. Native writes need not be transactional.
    fn write(&self, offset: u64, bytes: &[u8]);
}

impl<M: Memory + ?Sized> Memory for Rc<M> {
    fn size(&self) -> u64 {
        self.as_ref().size()
    }
    fn grow(&self, pages: u64) -> i64 {
        self.as_ref().grow(pages)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        self.as_ref().read(offset, bytes);
    }
    #[expect(
        unsafe_code,
        reason = "Forward the caller's admitted destination and initialization contract"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // SAFETY: the same destination and count retain the caller's guarantees.
        unsafe { self.as_ref().read_unsafe(offset, dst, count) }
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        self.as_ref().write(offset, bytes);
    }
}
impl<M: Memory + ?Sized> Memory for &M {
    fn size(&self) -> u64 {
        (**self).size()
    }
    fn grow(&self, pages: u64) -> i64 {
        (**self).grow(pages)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        (**self).read(offset, bytes);
    }
    #[expect(
        unsafe_code,
        reason = "Forward the caller's admitted destination and initialization contract"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // SAFETY: the same destination and count retain the caller's guarantees.
        unsafe { (**self).read_unsafe(offset, dst, count) }
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        (**self).write(offset, bytes);
    }
}

/// Shared native backing; clones identify the same physical bytes.
pub type VectorMemory = Rc<RefCell<Vec<u8>>>;
impl Memory for RefCell<Vec<u8>> {
    fn size(&self) -> u64 {
        self.borrow().len() as u64 / PAGE
    }
    fn grow(&self, pages: u64) -> i64 {
        let previous = self.size();
        let Some(length) = previous
            .checked_add(pages)
            .and_then(|value| value.checked_mul(PAGE))
            .and_then(|value| usize::try_from(value).ok())
        else {
            return -1;
        };
        let mut bytes = self.borrow_mut();
        let additional = length - bytes.len();
        if bytes.try_reserve(additional).is_err() {
            return -1;
        }
        bytes.resize(length, 0);
        i64::try_from(previous).expect("native page count")
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        let start = usize::try_from(offset).expect("native offset");
        let end = start
            .checked_add(bytes.len())
            .expect("native range overflow");
        bytes.copy_from_slice(&self.borrow()[start..end]);
    }
    #[expect(
        unsafe_code,
        reason = "Copy a fully admitted source into the caller's exclusive non-overlapping destination"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        let start = usize::try_from(offset).expect("native offset");
        let end = start.checked_add(count).expect("native range overflow");
        let backing = self.borrow();
        let source = &backing[start..end];
        // SAFETY: the source is admitted before copying. The caller guarantees
        // writable, non-overlapping space for every byte; u8 alignment is one.
        unsafe { std::ptr::copy_nonoverlapping(source.as_ptr(), dst, count) }
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        let start = usize::try_from(offset).expect("native offset");
        let end = start
            .checked_add(bytes.len())
            .expect("native range overflow");
        self.borrow_mut()[start..end].copy_from_slice(bytes);
    }
}

/// Native default backing for fixture and host integration.
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultMemoryImpl = VectorMemory;

///
/// `DefaultMemoryImpl`
///
/// Canister stable memory through IC Platform's owned system-call boundary.
///
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Default)]
pub struct DefaultMemoryImpl {
    _private: (),
}
#[cfg(target_arch = "wasm32")]
impl Memory for DefaultMemoryImpl {
    fn size(&self) -> u64 {
        ic_platform_runtime::ic0::stable64_size()
    }
    fn grow(&self, pages: u64) -> i64 {
        let previous = ic_platform_runtime::ic0::stable64_grow(pages);
        if previous == u64::MAX {
            -1
        } else {
            i64::try_from(previous).expect("IC page count")
        }
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        ic_platform_runtime::ic0::stable64_read(bytes, offset);
    }
    #[expect(
        unsafe_code,
        reason = "Bridge the caller's admitted raw destination to Platform's safe initialization helper"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // SAFETY: the caller admits the slice's validity and exclusivity;
        // MaybeUninit permits its original contents. Platform initializes the
        // complete destination or traps, satisfying the raw-read postcondition.
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(dst.cast::<std::mem::MaybeUninit<u8>>(), count)
        };
        ic_platform_runtime::ic0::stable64_read_uninit(bytes, offset);
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        ic_platform_runtime::ic0::stable64_write(bytes, offset);
    }
}

#[cfg(test)]
mod tests;
