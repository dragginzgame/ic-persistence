use crate::{LedgerCommitStore, constants::WASM_PAGE_SIZE_BYTES};
use ic_platform::stable_structures::{Memory, Storable, storable::Bound};
use std::borrow::Cow;
use thiserror::Error;

/// Stable-cell magic prefix written by `ic-stable-structures::Cell`.
pub const STABLE_CELL_MAGIC: &[u8; 3] = b"SCL";
/// Stable-cell layout version supported by this adapter.
pub const STABLE_CELL_LAYOUT_VERSION: u8 = 1;
/// Stable-cell header byte length.
pub const STABLE_CELL_HEADER_SIZE: usize = 8;
/// Byte offset where the stable-cell value payload starts.
pub const STABLE_CELL_VALUE_OFFSET: u64 = 8;

///
/// StableCellPayloadError
///
/// Stable-cell payload decode failure.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum StableCellPayloadError {
    /// Declared bytes exceed the current recovery ceiling before allocation.
    #[error("stable-cell ledger payload length {value_len} exceeds recovery limit")]
    TooLarge { value_len: u64 },
    /// Memory contents do not start with the stable-cell marker.
    #[error("memory is not an ic-stable-structures Cell")]
    NotStableCell,
    /// Stable-cell layout version does not match the adapter's current shape.
    #[error("unexpected stable-cell layout version {version}")]
    UnexpectedLayoutVersion {
        /// Observed stable-cell version.
        version: u8,
    },
    /// Stable-cell header length does not fit inside the memory.
    #[error("stable-cell payload length {value_len} exceeds available bytes {available_bytes}")]
    InvalidLength {
        /// Encoded value length.
        value_len: u64,
        /// Available payload bytes in memory.
        available_bytes: u64,
    },
}

///
/// StableCellLedgerError
///
/// Stable-cell ledger record validation failure.
///
/// Record errors retain the IC Wire decoder's typed source. Ledger recovery and
/// allocation integrity remain separate checks after decoding.
///

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum StableCellLedgerError {
    /// Stable-cell envelope is corrupt or unexpected.
    #[error(transparent)]
    Payload(#[from] StableCellPayloadError),
    /// Stable-cell value bytes are not a valid ledger record.
    #[error("stable-cell ledger record decode failed: {0}")]
    Record(#[source] ic_wire::Error),
}

/// Decode the raw value payload from an `ic-stable-structures::Cell` memory.
///
/// This helper is intentionally narrow: it recognizes the physical stable-cell
/// envelope and returns the value bytes. It does not deserialize those bytes or
/// decide whether they represent a valid allocation ledger.
pub fn decode_stable_cell_payload<M: Memory>(
    memory: &M,
) -> Result<Vec<u8>, StableCellPayloadError> {
    let value_len = stable_cell_payload_length(memory, crate::constants::MAX_LEDGER_RECORD_BYTES)?;
    let mut bytes = vec![0; value_len];
    memory.read(STABLE_CELL_VALUE_OFFSET, &mut bytes);
    Ok(bytes)
}

/// Validate a stable-cell envelope and return its payload length.
///
/// Reject empty/foreign memory, unsupported layouts, lengths beyond the backing
/// extent, and lengths above the caller's `maximum`. No payload is allocated or
/// read, and memory is never written or grown. This does not validate the value's
/// codec or schema. The ledger reader retains its separate recovery ceiling.
///
/// # Panics
/// Propagates backing `size`/`read` failures.
pub fn stable_cell_payload_length<M: Memory>(
    memory: &M,
    maximum: usize,
) -> Result<usize, StableCellPayloadError> {
    if memory.size() == 0 {
        return Err(StableCellPayloadError::NotStableCell);
    }

    let mut header = [0; STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut header);
    if &header[0..3] != STABLE_CELL_MAGIC {
        return Err(StableCellPayloadError::NotStableCell);
    }
    if header[3] != STABLE_CELL_LAYOUT_VERSION {
        return Err(StableCellPayloadError::UnexpectedLayoutVersion { version: header[3] });
    }

    let value_len = u64::from(u32::from_le_bytes([
        header[4], header[5], header[6], header[7],
    ]));
    let available_bytes = memory.size().saturating_mul(WASM_PAGE_SIZE_BYTES);
    let payload_capacity = available_bytes.saturating_sub(STABLE_CELL_VALUE_OFFSET);
    if value_len > payload_capacity {
        return Err(StableCellPayloadError::InvalidLength {
            value_len,
            available_bytes: payload_capacity,
        });
    }
    if value_len > maximum as u64 {
        return Err(StableCellPayloadError::TooLarge { value_len });
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the admitted caller ceiling is representable as usize"
    )]
    let value_len = value_len as usize;
    Ok(value_len)
}

/// Decode a `LedgerCommitStore` from stable-cell value bytes.
///
/// This decodes only the cell value payload, not the enclosing stable-cell
/// header. Use [`decode_stable_cell_payload`] first when inspecting raw stable
/// memory.
///
/// The returned record is decoded DTO state, not authority. The runtime
/// validates protected commit recovery before trusting any ledger payload.
pub fn decode_stable_cell_commit_store(bytes: &[u8]) -> Result<LedgerCommitStore, ic_wire::Error> {
    use crate::encoding::{RECORD_LIMITS, RECORD_MAGIC};
    if bytes.len() > crate::constants::MAX_LEDGER_RECORD_BYTES {
        return Err(ic_wire::Error::Limit);
    }
    let payload = bytes
        .strip_prefix(RECORD_MAGIC)
        .ok_or(ic_wire::Error::Malformed)?;
    ic_wire::native::decode(payload, RECORD_LIMITS)
}

/// Fallibly decode a ledger record from its stable-cell envelope and value.
///
/// Empty memory returns an uninitialized record without writing a cell. Nonempty
/// memory must pass the current envelope, byte bounds and record decoding. The
/// returned DTO still requires protected ledger recovery before it is authority.
/// The runtime retains this decoded record for recovery and separately prepares
/// bounded publication bytes; reading never initializes a Cell.
pub fn decode_stable_cell_commit_store_from_memory<M: Memory>(
    memory: &M,
) -> Result<LedgerCommitStore, StableCellLedgerError> {
    if memory.size() == 0 {
        return Ok(LedgerCommitStore::default());
    }

    let payload = decode_stable_cell_payload(memory)?;
    decode_stable_cell_commit_store(&payload).map_err(StableCellLedgerError::Record)
}

///
/// EncodedLedgerRecord
///
/// Temporary publication bytes, never recovered authority or retained runtime state.
///

pub struct EncodedLedgerRecord {
    bytes: Vec<u8>,
}

impl EncodedLedgerRecord {
    pub(crate) fn new(record: &LedgerCommitStore) -> Result<Self, ic_wire::Error> {
        use crate::encoding::{RECORD_LIMITS, RECORD_MAGIC};
        let payload = ic_wire::native::encode(record, RECORD_LIMITS)?;
        let mut bytes = Vec::with_capacity(RECORD_MAGIC.len() + payload.len());
        bytes.extend_from_slice(RECORD_MAGIC);
        bytes.extend_from_slice(&payload);
        Ok(Self { bytes })
    }

    pub(crate) const fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl Storable for EncodedLedgerRecord {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.bytes)
    }
    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    fn from_bytes(_: Cow<'_, [u8]>) -> Self {
        panic!("publication bytes cannot recover ledger authority")
    }
}

#[cfg(test)]
mod tests;
