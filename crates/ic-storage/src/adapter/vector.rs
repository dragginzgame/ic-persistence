//! Owned fixed-stride slots with bounded, checksummed values and a Wire root.
use super::{
    Memory, Storable,
    error::{CollectionError as Error, Result, ensure},
    record,
};
use crate::{DefaultMemoryImpl, RuntimeMemory, hash};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::Cell, marker::PhantomData};

const MAGIC: [u8; 8] = *b"ICVECC01";
const DATA: u64 = record::HEADER_BYTES;
const SLOT_HEADER: usize = 12;

#[derive(Clone, Serialize, Deserialize)]
struct Root {
    length: u64,
    bound: (u32, bool),
}

///
/// `StableVec`
///
/// Indexed stable values in fixed-stride slots. Values require a finite bound.
/// Each slot protects its index, length and payload with an integrity checksum.
/// Native interrupted writes poison the owner and may damage retained bytes.
///
pub struct StableVec<T: Storable, M: Memory = DefaultMemoryImpl> {
    memory: RuntimeMemory<M>,
    length: Cell<u64>,
    poisoned: Cell<bool>,
    record: PhantomData<T>,
}

impl<T: Storable, M: Memory> StableVec<T, M> {
    /// Create only in an empty allocation. Invalid bounds refuse before writes.
    ///
    /// # Errors
    /// Rejects nonempty memory, invalid encoding/bounds or capacity refusal before writing.
    pub fn create(memory: RuntimeMemory<M>) -> Result<Self> {
        ensure(memory.size() == 0, Error::NotEmpty)?;
        Self::reset(memory)
    }
    /// Explicitly discard previous contents in this allocation.
    ///
    /// # Errors
    /// Rejects invalid encoding/bounds or capacity refusal before replacing contents.
    pub fn reset(memory: RuntimeMemory<M>) -> Result<Self> {
        let bytes = Self::header(0)?;
        record::reserve(&memory, DATA)?;
        memory.write(0, &bytes);
        Ok(Self {
            memory,
            length: Cell::new(0),
            poisoned: Cell::new(false),
            record: PhantomData,
        })
    }
    /// Read and validate the current root without initializing or writing.
    ///
    /// # Errors
    /// Rejects foreign, corrupt or oversized records and incompatible codec bounds.
    pub fn load(memory: RuntimeMemory<M>) -> Result<Self> {
        let root: Root = record::read(&memory, 0, DATA, MAGIC)?;
        ensure(
            root.bound == Self::bound()?,
            Error::Corrupt("vector codec bound changed"),
        )?;
        ensure(
            Self::end(root.length)?
                <= memory
                    .size()
                    .checked_mul(crate::WASM_PAGE_SIZE_BYTES)
                    .ok_or(Error::Budget)?,
            Error::Corrupt("vector length exceeds memory"),
        )?;
        Ok(Self {
            memory,
            length: Cell::new(root.length),
            poisoned: Cell::new(false),
            record: PhantomData,
        })
    }
    fn bound() -> Result<(u32, bool)> {
        record::bound::<T>().ok_or(Error::Corrupt("vector requires a finite value bound"))
    }
    fn end(length: u64) -> Result<u64> {
        let stride = u64::from(Self::bound()?.0) + SLOT_HEADER as u64;
        DATA.checked_add(length.checked_mul(stride).ok_or(Error::Budget)?)
            .ok_or(Error::Budget)
    }
    fn header(length: u64) -> Result<Vec<u8>> {
        let bytes = record::encode(
            MAGIC,
            &Root {
                length,
                bound: Self::bound()?,
            },
        )?;
        ensure(bytes.len() as u64 <= DATA, Error::Budget)?;
        Ok(bytes)
    }
    fn healthy(&self) -> Result<()> {
        ensure(!self.poisoned.get(), Error::Poisoned)
    }

    /// Decode an indexed value, checking length and checksum before its codec.
    ///
    /// # Errors
    /// Reports structural, integrity or application decoding failures without writing.
    ///
    /// # Panics
    /// Propagates backing panics or a faulty caller codec.
    pub fn get(&self, index: u64) -> Result<Option<T>> {
        let mut bytes = Vec::new();
        if !self.read_slot(index, &mut bytes)? {
            return Ok(None);
        }
        Ok(Some(T::from_bytes(Cow::Owned(bytes))?))
    }
    fn read_slot(&self, index: u64, bytes: &mut Vec<u8>) -> Result<bool> {
        self.healthy()?;
        if index >= self.length.get() {
            return Ok(false);
        }
        let offset = Self::end(index)?;
        let (maximum, fixed) = Self::bound()?;
        let mut header = [0; SLOT_HEADER];
        if fixed {
            let size = SLOT_HEADER
                .checked_add(maximum as usize)
                .ok_or(Error::Budget)?;
            bytes.resize(size, 0);
            self.memory.read(offset, bytes);
            header.copy_from_slice(&bytes[..SLOT_HEADER]);
            bytes.copy_within(SLOT_HEADER.., 0);
            bytes.truncate(maximum as usize);
        } else {
            self.memory.read(offset, &mut header);
        }
        let length = u32::from_le_bytes(header[..4].try_into().unwrap());
        ensure(
            length <= maximum && (!fixed || length == maximum),
            Error::Corrupt("vector slot length exceeds bound"),
        )?;
        if !fixed {
            bytes.resize(length as usize, 0);
            self.memory.read(offset + SLOT_HEADER as u64, bytes);
        }
        let digest = u64::from_le_bytes(header[4..].try_into().unwrap());
        ensure(
            checksum(index, &header[..4], bytes) == digest,
            Error::Checksum,
        )?;
        T::check_bounds(bytes)?;
        Ok(true)
    }
    /// Replace an existing value. Encoding and bounds admission precede writes.
    ///
    /// # Errors
    /// Rejects invalid indices, encoding, bounds or capacity refusal before writes.
    pub fn set(&self, index: u64, item: &T) -> Result<()> {
        self.healthy()?;
        ensure(index < self.length.get(), Error::Bounds)?;
        let bytes = item.to_bytes_checked()?;
        self.write_slot(index, &bytes)
    }
    fn write_slot(&self, index: u64, bytes: &[u8]) -> Result<()> {
        let offset = Self::end(index)?;
        let mut header = [0; SLOT_HEADER];
        let length = u32::try_from(bytes.len()).map_err(|_| Error::Budget)?;
        header[..4].copy_from_slice(&length.to_le_bytes());
        let digest = checksum(index, &header[..4], bytes);
        header[4..].copy_from_slice(&digest.to_le_bytes());
        self.poisoned.set(true);
        self.memory.write(offset + SLOT_HEADER as u64, bytes);
        self.memory.write(offset, &header);
        self.poisoned.set(false);
        Ok(())
    }
    /// Append after reserving capacity; publish the new length last.
    ///
    /// # Errors
    /// Rejects invalid encoding, bounds, arithmetic overflow or capacity refusal before writes.
    pub fn push(&self, item: &T) -> Result<()> {
        self.healthy()?;
        let bytes = item.to_bytes_checked()?;
        let length = self.length.get().checked_add(1).ok_or(Error::Budget)?;
        let header = Self::header(length)?;
        record::reserve(&self.memory, Self::end(length)?)?;
        self.write_slot(length - 1, &bytes)?;
        self.poisoned.set(true);
        self.memory.write(0, &header);
        self.length.set(length);
        self.poisoned.set(false);
        Ok(())
    }
    /// Remove the final value, retaining allocated capacity for reuse.
    ///
    /// # Errors
    /// Rejects integrity, decoding or structural errors before changing the length.
    pub fn pop(&self) -> Result<Option<T>> {
        self.healthy()?;
        let Some(length) = self.length.get().checked_sub(1) else {
            return Ok(None);
        };
        let value = self.get(length)?;
        let header = Self::header(length)?;
        self.poisoned.set(true);
        self.memory.write(0, &header);
        self.length.set(length);
        self.poisoned.set(false);
        Ok(value)
    }
    /// Number of committed slots. A poisoned owner must be dropped.
    ///
    /// # Panics
    /// A poisoned owner must be dropped before further observation.
    pub fn len(&self) -> u64 {
        self.healthy().expect("vector owner is poisoned");
        self.length.get()
    }
    /// Whether the vector has no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Decode slots lazily, returning errors without panicking on malformed data.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Result<T>> + '_ {
        let mut bytes = Vec::new();
        (0..self.len()).map(move |index| {
            ensure(self.read_slot(index, &mut bytes)?, Error::Bounds)?;
            Ok(T::from_bytes(Cow::Borrowed(&bytes))?)
        })
    }
}

fn checksum(index: u64, length: &[u8], bytes: &[u8]) -> u64 {
    let digest = hash::fnv64(hash::FNV_OFFSET, &index.to_le_bytes());
    hash::fnv64(hash::fnv64(digest, length), bytes)
}
