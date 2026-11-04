//! Bounded, integrity-checked Wire records shared by owned collection engines.
use super::{
    Memory,
    error::{CollectionError as Error, Result, ensure},
};
use crate::{WASM_PAGE_SIZE_BYTES as PAGE, hash};
use serde::{Serialize, de::DeserializeOwned};

pub const HEADER_BYTES: u64 = 4096;
pub const MAX_RECORD: usize = 16 * 1024 * 1024;
pub(super) const PREFIX: usize = 20;
const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: MAX_RECORD,
    allocation: 64 * 1024 * 1024,
    nodes: 1_048_576,
    depth: 16,
};

pub fn encode<T: Serialize>(magic: [u8; 8], value: &T) -> Result<Vec<u8>> {
    let payload = ic_wire::native::encode(value, LIMITS)?;
    encode_payload(magic, &payload)
}

pub fn encode_payload(magic: [u8; 8], payload: &[u8]) -> Result<Vec<u8>> {
    encode_payload_with(magic, payload.len(), |bytes| {
        bytes.extend_from_slice(payload);
        Ok(())
    })
}

// Append directly into the final frame, then seal its admitted payload. The
// caller supplies the exact size so node serialization needs only one buffer.
pub fn encode_payload_with(
    magic: [u8; 8],
    length: usize,
    write: impl FnOnce(&mut Vec<u8>) -> Result<()>,
) -> Result<Vec<u8>> {
    ensure(length <= MAX_RECORD, Error::Budget)?;
    let mut bytes = Vec::with_capacity(PREFIX + length);
    bytes.resize(PREFIX, 0);
    write(&mut bytes)?;
    ensure(
        bytes.len() == PREFIX + length,
        Error::Corrupt("record payload length"),
    )?;
    let digest = hash::fnv64(hash::FNV_OFFSET, &bytes[PREFIX..]);
    bytes[..8].copy_from_slice(&magic);
    bytes[8..12].copy_from_slice(
        &u32::try_from(length)
            .map_err(|_| Error::Budget)?
            .to_le_bytes(),
    );
    bytes[12..PREFIX].copy_from_slice(&digest.to_le_bytes());
    Ok(bytes)
}

pub fn decode<T: DeserializeOwned>(magic: [u8; 8], bytes: &[u8]) -> Result<T> {
    Ok(ic_wire::native::decode(
        decode_payload(magic, bytes)?,
        LIMITS,
    )?)
}

pub fn decode_payload(magic: [u8; 8], bytes: &[u8]) -> Result<&[u8]> {
    ensure(bytes.len() >= PREFIX && bytes[..8] == magic, Error::Format)?;
    let length = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    ensure(
        length <= MAX_RECORD && bytes.len() == PREFIX + length,
        Error::Budget,
    )?;
    let payload = &bytes[PREFIX..];
    check_digest(&bytes[..PREFIX], payload)?;
    Ok(payload)
}

fn check_digest(prefix: &[u8], payload: &[u8]) -> Result<()> {
    let digest = u64::from_le_bytes(prefix[12..20].try_into().unwrap());
    ensure(
        hash::fnv64(hash::FNV_OFFSET, payload) == digest,
        Error::Checksum,
    )
}

pub fn read<T: DeserializeOwned>(
    memory: &impl Memory,
    offset: u64,
    capacity: u64,
    magic: [u8; 8],
) -> Result<T> {
    let bytes = read_bytes(memory, offset, capacity, magic)?;
    decode(magic, &bytes)
}

pub fn read_bytes(
    memory: &impl Memory,
    offset: u64,
    capacity: u64,
    magic: [u8; 8],
) -> Result<Vec<u8>> {
    let (prefix, length) = read_prefix(memory, offset, capacity, magic)?;
    let mut bytes = vec![0; PREFIX + length];
    bytes[..PREFIX].copy_from_slice(&prefix);
    memory.read(offset + PREFIX as u64, &mut bytes[PREFIX..]);
    Ok(bytes)
}

// External values need an owned payload, not an owned frame followed by a
// second allocation/copy. Keep admission and checksum policy shared with frames.
pub fn read_payload(
    memory: &impl Memory,
    offset: u64,
    capacity: u64,
    magic: [u8; 8],
) -> Result<Vec<u8>> {
    let (prefix, length) = read_prefix(memory, offset, capacity, magic)?;
    let mut payload = vec![0; length];
    memory.read(offset + PREFIX as u64, &mut payload);
    check_digest(&prefix, &payload)?;
    Ok(payload)
}

fn read_prefix(
    memory: &impl Memory,
    offset: u64,
    capacity: u64,
    magic: [u8; 8],
) -> Result<([u8; PREFIX], usize)> {
    ensure(capacity >= PREFIX as u64, Error::Budget)?;
    let end = offset.checked_add(capacity).ok_or(Error::Budget)?;
    ensure(
        end <= memory.size().checked_mul(PAGE).ok_or(Error::Budget)?,
        Error::Corrupt("record outside memory"),
    )?;
    let mut prefix = [0; PREFIX];
    memory.read(offset, &mut prefix);
    ensure(prefix[..8] == magic, Error::Format)?;
    let length = u32::from_le_bytes(prefix[8..12].try_into().unwrap()) as usize;
    ensure(
        length <= MAX_RECORD && (PREFIX + length) as u64 <= capacity,
        Error::Budget,
    )?;
    Ok((prefix, length))
}

pub fn reserve(memory: &impl Memory, bytes: u64) -> Result<()> {
    let pages = bytes.div_ceil(PAGE);
    let current = memory.size();
    if pages > current {
        ensure(memory.grow(pages - current) != -1, Error::Growth)?;
    }
    Ok(())
}

pub fn bound<T: super::Storable>() -> Option<(u32, bool)> {
    match T::BOUND {
        super::Bound::Unbounded => None,
        super::Bound::Bounded {
            max_size,
            is_fixed_size,
        } => Some((max_size, is_fixed_size)),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
