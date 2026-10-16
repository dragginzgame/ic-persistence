use super::{Config, Error};
use crate::Memory;

#[cfg(test)]
mod tests;

pub(super) const SIZE: usize = 4096;
pub(super) const PAYLOAD: usize = SIZE - 24;
pub(super) const DEGREE: usize = 16;
pub(super) const MAX: usize = 2 * DEGREE - 1;
pub(super) const MAX_HEIGHT: usize = 64;
pub(super) const FREE: u8 = 1;
pub(super) const BLOB: u8 = 2;
const LEAF: u8 = 3;
const BRANCH: u8 = 4;
pub(super) type Page = [u8; SIZE];

#[derive(Clone, Copy)]
pub(super) struct Header {
    pub root: u64,
    pub len: u64,
    pub next: u64,
    pub free: u64,
    pub config: Config,
    pub identity: [u8; 16],
}
impl Header {
    pub const fn new(config: Config, identity: [u8; 16]) -> Self {
        Self {
            root: 0,
            len: 0,
            next: 1,
            free: 0,
            config,
            identity,
        }
    }
    pub fn encode(self) -> Page {
        let mut page = [0; SIZE];
        page[..8].copy_from_slice(b"ICBTREE1");
        put(&mut page, 8, self.root);
        put(&mut page, 16, self.len);
        put(&mut page, 24, self.next);
        put(&mut page, 32, self.free);
        put(&mut page, 40, u64::from(self.config.max_key_bytes));
        put(&mut page, 48, u64::from(self.config.max_value_bytes));
        page[56..72].copy_from_slice(&self.identity);
        seal(&mut page);
        page
    }
    pub fn read(memory: &impl Memory) -> Result<Self, Error> {
        if memory.size() == 0 {
            return Err(Error::Format);
        }
        let mut page = [0; SIZE];
        memory.read(0, &mut page);
        if &page[..8] != b"ICBTREE1" {
            return Err(Error::Format);
        }
        verify(&page)?;
        let config = Config {
            max_key_bytes: u32::try_from(number(&page, 40)).map_err(|_| Error::Corrupt)?,
            max_value_bytes: u32::try_from(number(&page, 48)).map_err(|_| Error::Corrupt)?,
        };
        config.check().map_err(|_| Error::Corrupt)?;
        let mut identity = [0; 16];
        identity.copy_from_slice(&page[56..72]);
        let header = Self {
            root: number(&page, 8),
            len: number(&page, 16),
            next: number(&page, 24),
            free: number(&page, 32),
            config,
            identity,
        };
        if header.root == 0
            || header.root >= header.next
            || header.free >= header.next
            || header.next.checked_mul(SIZE as u64).ok_or(Error::Corrupt)?
                > memory.size().checked_mul(65_536).ok_or(Error::Corrupt)?
        {
            return Err(Error::Corrupt);
        }
        Ok(header)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Record {
    pub key: u64,
    pub key_len: u64,
    pub value: u64,
    pub value_len: u64,
    pub key_inline: [u8; 32],
    pub value_inline: [u8; 16],
}
#[derive(Clone)]
pub(super) struct Node {
    pub entries: Vec<Record>,
    pub children: Vec<u64>,
}
impl Node {
    pub const fn leaf() -> Self {
        Self {
            entries: Vec::new(),
            children: Vec::new(),
        }
    }
    pub fn encode(&self) -> Result<Page, Error> {
        if self.entries.len() > MAX
            || (!self.children.is_empty() && self.children.len() != self.entries.len() + 1)
        {
            return Err(Error::Corrupt);
        }
        let mut page = [0; SIZE];
        page[0] = if self.children.is_empty() {
            LEAF
        } else {
            BRANCH
        };
        put(&mut page, 8, self.entries.len() as u64);
        for (i, entry) in self.entries.iter().enumerate() {
            let at = 16 + i * 80;
            put(&mut page, at, entry.key);
            put(&mut page, at + 8, entry.key_len);
            put(&mut page, at + 16, entry.value);
            put(&mut page, at + 24, entry.value_len);
            page[at + 32..at + 64].copy_from_slice(&entry.key_inline);
            page[at + 64..at + 80].copy_from_slice(&entry.value_inline);
        }
        for (i, child) in self.children.iter().enumerate() {
            put(&mut page, 16 + MAX * 80 + i * 8, *child);
        }
        seal(&mut page);
        Ok(page)
    }
    pub fn decode(page: &Page, next: u64, config: Config) -> Result<Self, Error> {
        if page[0] != LEAF && page[0] != BRANCH {
            return Err(Error::Corrupt);
        }
        let count = usize::try_from(number(page, 8)).map_err(|_| Error::Corrupt)?;
        if count > MAX {
            return Err(Error::Corrupt);
        }
        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let at = 16 + i * 80;
            let mut key_inline = [0; 32];
            let mut value_inline = [0; 16];
            key_inline.copy_from_slice(&page[at + 32..at + 64]);
            value_inline.copy_from_slice(&page[at + 64..at + 80]);
            let entry = Record {
                key_inline,
                value_inline,
                key: number(page, at),
                key_len: number(page, at + 8),
                value: number(page, at + 16),
                value_len: number(page, at + 24),
            };
            if entry.key_len > u64::from(config.max_key_bytes)
                || entry.value_len > u64::from(config.max_value_bytes)
                || entry.key >= next
                || entry.value >= next
                || (entry.key == 0) != (entry.key_len <= 32)
                || (entry.value == 0) != (entry.value_len <= 16)
            {
                return Err(Error::Corrupt);
            }
            entries.push(entry);
        }
        let mut children = Vec::new();
        if page[0] == BRANCH {
            for i in 0..=count {
                let child = number(page, 16 + MAX * 80 + i * 8);
                if child == 0 || child >= next {
                    return Err(Error::Corrupt);
                }
                children.push(child);
            }
        }
        Ok(Self { entries, children })
    }
}

pub(super) fn put(page: &mut Page, at: usize, value: u64) {
    page[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn number(page: &Page, at: usize) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&page[at..at + 8]);
    u64::from_le_bytes(bytes)
}
fn checksum(page: &Page) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    // The covered region is a multiple of eight. Fixed-size inner iterations
    // let Wasm eliminate per-byte loop control without changing FNV-1a order.
    for chunk in page[..SIZE - 8].chunks_exact(8) {
        for byte in chunk {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
    }
    hash
}
pub(super) fn seal(page: &mut Page) {
    put(page, SIZE - 8, checksum(page));
}
pub(super) fn free(next: u64) -> Page {
    const TAIL_FACTOR: u64 = {
        let mut factor = 1u64;
        let mut at = 16;
        while at < SIZE - 8 {
            factor = factor.wrapping_mul(0x100_0000_01b3);
            at += 1;
        }
        factor
    };
    let mut page = [0; SIZE];
    page[0] = FREE;
    put(&mut page, 8, next);
    // Only freshly constructed free pages have a known-zero tail. FNV-1a
    // over each zero byte multiplies by the prime, so fold the tail at once.
    // Reads still verify every byte, including the reserved zero padding.
    let prefix = page[..16]
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
        });
    put(&mut page, SIZE - 8, prefix.wrapping_mul(TAIL_FACTOR));
    page
}
fn verify(page: &Page) -> Result<(), Error> {
    if number(page, SIZE - 8) != checksum(page) {
        return Err(Error::Corrupt);
    }
    Ok(())
}
pub(super) fn read(memory: &impl Memory, next: u64, id: u64) -> Result<Page, Error> {
    if id == 0 || id >= next {
        return Err(Error::Corrupt);
    }
    let offset = id.checked_mul(SIZE as u64).ok_or(Error::Corrupt)?;
    if offset.checked_add(SIZE as u64).ok_or(Error::Corrupt)?
        > memory.size().checked_mul(65_536).ok_or(Error::Corrupt)?
    {
        return Err(Error::Corrupt);
    }
    let mut page = [0; SIZE];
    memory.read(offset, &mut page);
    verify(&page)?;
    Ok(page)
}
pub(super) fn blob(
    memory: &impl Memory,
    next: u64,
    start: u64,
    len: u64,
) -> Result<Vec<u8>, Error> {
    let len = usize::try_from(len).map_err(|_| Error::Corrupt)?;
    let mut output = Vec::with_capacity(len);
    let mut id = start;
    while output.len() < len {
        let page = read(memory, next, id)?;
        if page[0] != BLOB {
            return Err(Error::Corrupt);
        }
        let take = (len - output.len()).min(PAYLOAD);
        output.extend_from_slice(&page[16..16 + take]);
        id = number(&page, 8);
    }
    if id != 0 {
        return Err(Error::Corrupt);
    }
    Ok(output)
}
