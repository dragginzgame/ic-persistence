use crate::{Memory, WASM_PAGE_SIZE_BYTES};
use thiserror::Error;

/// Stable-cell magic prefix written by `Storage Cell`.
pub const STABLE_CELL_MAGIC: &[u8; 8] = b"ICCELL01";
/// Stable-cell header byte length.
pub const STABLE_CELL_HEADER_SIZE: usize = 24;
/// Byte offset where the stable-cell value payload starts.
pub const STABLE_CELL_VALUE_OFFSET: u64 = STABLE_CELL_HEADER_SIZE as u64;

#[expect(
    clippy::empty_line_after_doc_comments,
    reason = "Repository rustdoc style separates item docs from attributes."
)]
///
/// `StableCellPayloadError`
///
/// Stable-cell payload decode failure.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum StableCellPayloadError {
    /// Declared bytes exceed the current recovery ceiling before allocation.
    #[error("stable-cell payload length {value_len} exceeds the caller's limit")]
    TooLarge { value_len: u64 },
    /// Memory contents do not start with the stable-cell marker.
    #[error("memory is not a Storage Cell")]
    NotStableCell,
    /// Stable-cell header length does not fit inside the memory.
    #[error("stable-cell payload length {value_len} exceeds available bytes {available_bytes}")]
    InvalidLength {
        /// Encoded value length.
        value_len: u64,
        /// Available payload bytes in memory.
        available_bytes: u64,
    },
}

/// Validate a stable-cell envelope and return its payload length.
///
/// Reject empty/foreign memory, unsupported layouts, lengths beyond the backing
/// extent, and lengths above the caller's `maximum`. No payload is allocated or
/// read, and memory is never written or grown. This does not validate the value's
/// codec or schema.
///
/// # Panics
/// Propagates backing `size`/`read` failures.
///
/// # Errors
/// Rejects foreign or unsupported envelopes and lengths exceeding the backing extent or caller limit.
pub fn stable_cell_payload_length<M: Memory>(
    memory: &M,
    maximum: usize,
) -> Result<usize, StableCellPayloadError> {
    if memory.size() == 0 {
        return Err(StableCellPayloadError::NotStableCell);
    }

    let mut header = [0; STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut header);
    if &header[..8] != STABLE_CELL_MAGIC || header[12..16].iter().any(|&byte| byte != 0) {
        return Err(StableCellPayloadError::NotStableCell);
    }
    let value_len = u64::from(u32::from_le_bytes(header[8..12].try_into().unwrap()));
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
