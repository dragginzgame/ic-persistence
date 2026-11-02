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
/// DefaultMemoryImpl
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
        ic_platform_runtime::cdk::stable::stable_grow(pages).map_or(-1, |previous| {
            i64::try_from(previous).expect("IC page count")
        })
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        ic_platform_runtime::cdk::stable::stable_read(offset, bytes);
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        ic_platform_runtime::cdk::stable::stable_write(offset, bytes);
    }
}
