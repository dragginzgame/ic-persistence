//! Governed single-value storage over an exclusive allocation lease.
use super::{
    Bound, Memory, Storable,
    error::{CollectionError as Error, Result, ensure},
};
use crate::stable_cell::{STABLE_CELL_MAGIC, STABLE_CELL_VALUE_OFFSET};
use crate::{
    DefaultMemoryImpl, RuntimeMemory, STABLE_CELL_HEADER_SIZE, WASM_PAGE_SIZE_BYTES, hash,
};
use std::borrow::Cow;

///
/// `StableCell`
///
/// A cached, checksummed value persisted in an exclusively leased allocation.
/// Loading is read-only and requires a caller-selected payload byte ceiling.
///
pub struct StableCell<T: Storable, M: Memory = DefaultMemoryImpl> {
    memory: RuntimeMemory<M>,
    value: T,
}

impl<T: Storable, M: Memory> StableCell<T, M> {
    /// Create in empty memory, admitting encoding and bounds before writes.
    ///
    /// # Errors
    /// Rejects nonempty memory, invalid encoding/bounds or capacity refusal before writing.
    pub fn create(memory: RuntimeMemory<M>, value: T) -> Result<Self> {
        ensure(memory.size() == 0, Error::NotEmpty)?;
        Self::reset(memory, value)
    }

    /// Explicitly replace contents. Backing failures retain host rollback semantics.
    ///
    /// # Errors
    /// Rejects invalid encoding/bounds or capacity refusal before replacing contents.
    pub fn reset(memory: RuntimeMemory<M>, value: T) -> Result<Self> {
        Self::write(&memory, &value)?;
        Ok(Self { memory, value })
    }

    /// Load without writing. Admit the smaller of `maximum` and the codec bound
    /// before allocating, then verify integrity before decoding application data.
    ///
    /// # Errors
    /// Rejects foreign, corrupt or oversized records and incompatible codec bounds.
    ///
    /// # Panics
    /// Propagates backing panics or a faulty caller codec; no native rollback is provided.
    pub fn load(memory: RuntimeMemory<M>, maximum: usize) -> Result<Self> {
        let maximum = match T::BOUND {
            Bound::Bounded { max_size, .. } => maximum.min(max_size as usize),
            Bound::Unbounded => maximum,
        };
        let length = crate::stable_cell_payload_length(&memory, maximum)?;
        let mut header = [0; STABLE_CELL_HEADER_SIZE];
        memory.read(0, &mut header);
        let mut bytes = vec![0; length];
        memory.read(STABLE_CELL_VALUE_OFFSET, &mut bytes);
        let expected = u64::from_le_bytes(header[16..24].try_into().unwrap());
        ensure(checksum(&header, &bytes) == expected, Error::Checksum)?;
        T::check_bounds(&bytes)?;
        let value = T::from_bytes(Cow::Owned(bytes))?;
        Ok(Self { memory, value })
    }

    /// Borrow the cached value without IO or decoding.
    pub const fn get(&self) -> &T {
        &self.value
    }

    /// Persist a replacement and return the old value. Admission and growth
    /// refusal preserve existing bytes. Native interrupted writes are not atomic.
    ///
    /// # Errors
    /// Rejects invalid indices, encoding, bounds or capacity refusal before writes.
    pub fn set(&mut self, value: T) -> Result<T> {
        Self::write(&self.memory, &value)?;
        Ok(std::mem::replace(&mut self.value, value))
    }

    fn write(memory: &RuntimeMemory<M>, value: &T) -> Result<()> {
        let bytes = value.to_bytes_checked()?;
        let length = u32::try_from(bytes.len()).map_err(|_| Error::Budget)?;
        let pages = (STABLE_CELL_VALUE_OFFSET + u64::from(length)).div_ceil(WASM_PAGE_SIZE_BYTES);
        let current = memory.size();
        if pages > current {
            memory.grow(pages - current).map_err(|_| Error::Growth)?;
        }
        let mut header = [0; STABLE_CELL_HEADER_SIZE];
        header[..8].copy_from_slice(STABLE_CELL_MAGIC);
        header[8..12].copy_from_slice(&length.to_le_bytes());
        let digest = checksum(&header, &bytes);
        header[16..24].copy_from_slice(&digest.to_le_bytes());
        memory.write(STABLE_CELL_VALUE_OFFSET, &bytes);
        memory.write(0, &header);
        Ok(())
    }
}

fn checksum(header: &[u8], payload: &[u8]) -> u64 {
    hash::fnv64(hash::fnv64(hash::FNV_OFFSET, &header[..16]), payload)
}
