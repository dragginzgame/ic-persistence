//! IC Wire owns bounded serialization; Logging owns restored-state admission.

use std::num::NonZeroUsize;

use ic_logging::{Entry, Level, Limits, Log, LogSnapshot};
use ic_wire::{List, native};
use serde::{Deserialize, Serialize};

use crate::{LogIdentity, MAX_ENTRIES, MAX_RECORD_BYTES, StoreError};

const MAGIC: &[u8] = b"ICL\x02";
// These are conservative Serde traversal allowances, not measured heap sizes.
// Static row bounds and borrowed text keep decoding bounded before restoration.
const CODEC_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: MAX_RECORD_BYTES - MAGIC.len(),
    allocation: 4 * 1024 * 1024,
    nodes: 64 + MAX_ENTRIES * 64,
    depth: 16,
};

#[derive(Deserialize, Serialize)]
struct Checkpoint<'a> {
    epoch: u64,
    #[serde(with = "StoredLimits")]
    limits: Limits,
    next_sequence: Option<u64>,
    last_created_at_secs: Option<u64>,
    #[serde(borrow)]
    entries: List<StoredEntry<'a>, MAX_ENTRIES>,
}

#[derive(Deserialize, Serialize)]
struct StoredEntry<'a> {
    sequence: u64,
    created_at_secs: u64,
    #[serde(with = "StoredLevel")]
    level: Level,
    truncated: bool,
    source: &'a str,
    #[serde(borrow)]
    topic: Option<&'a str>,
    message: &'a str,
}

// Remote derives describe Persistence's schema without adding serialization to
// the dependency-free Logging engine or duplicating its semantic admission.
#[derive(Deserialize, Serialize)]
#[serde(remote = "Limits")]
#[expect(
    clippy::struct_field_names,
    reason = "Serde remote derives must match the canonical Logging limit field names"
)]
struct StoredLimits {
    max_entries: NonZeroUsize,
    max_message_bytes: usize,
    max_label_bytes: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "Level")]
enum StoredLevel {
    Debug,
    Info,
    Ok,
    Warn,
    Error,
}

pub fn encode(identity: LogIdentity, state: &LogSnapshot) -> Result<Vec<u8>, StoreError> {
    let entries = state
        .entries
        .iter()
        .map(|entry| StoredEntry {
            sequence: entry.sequence,
            created_at_secs: entry.created_at_secs,
            level: entry.level,
            truncated: entry.truncated,
            source: &entry.source,
            topic: entry.topic.as_deref(),
            message: &entry.message,
        })
        .collect();
    let checkpoint = Checkpoint {
        epoch: identity.epoch,
        limits: state.limits,
        next_sequence: state.next_sequence,
        last_created_at_secs: state.last_created_at_secs,
        entries: List::new(entries)?,
    };
    let payload = native::encode(&checkpoint, CODEC_LIMITS).map_err(|error| match error {
        ic_wire::Error::Limit => StoreError::RecordTooLarge,
        other => StoreError::Codec(other),
    })?;
    let mut bytes = Vec::with_capacity(MAGIC.len() + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

pub fn decode(bytes: &[u8], identity: LogIdentity, limits: Limits) -> Result<Log, StoreError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(StoreError::RecordTooLarge);
    }
    let payload = bytes
        .strip_prefix(MAGIC)
        .ok_or(StoreError::UnsupportedFormat)?;
    let checkpoint: Checkpoint<'_> =
        native::decode(payload, CODEC_LIMITS).map_err(|error| match error {
            ic_wire::Error::Malformed | ic_wire::Error::Trailing => StoreError::MalformedRecord,
            other => StoreError::Codec(other),
        })?;
    if checkpoint.epoch != identity.epoch {
        return Err(StoreError::IdentityMismatch);
    }
    if checkpoint.limits != limits {
        return Err(StoreError::LimitsMismatch);
    }
    if checkpoint.entries.as_slice().len() > limits.max_entries.get() {
        return Err(StoreError::InvalidState(
            ic_logging::RestoreError::TooManyEntries,
        ));
    }
    // Text borrows admitted input bytes. Apply the caller's field limits before
    // constructing owned Logging entries; canonical restoration follows once.
    for entry in checkpoint.entries.as_slice() {
        if entry.source.len() > limits.max_label_bytes
            || entry
                .topic
                .is_some_and(|topic| topic.len() > limits.max_label_bytes)
            || entry.message.len() > limits.max_message_bytes
        {
            return Err(StoreError::MalformedRecord);
        }
    }
    let entries = checkpoint
        .entries
        .into_vec()
        .into_iter()
        .map(|entry| Entry {
            sequence: entry.sequence,
            created_at_secs: entry.created_at_secs,
            level: entry.level,
            truncated: entry.truncated,
            source: entry.source.to_owned(),
            topic: entry.topic.map(str::to_owned),
            message: entry.message.to_owned(),
        })
        .collect();
    Log::restore(LogSnapshot {
        limits,
        entries,
        next_sequence: checkpoint.next_sequence,
        last_created_at_secs: checkpoint.last_created_at_secs,
    })
    .map_err(Into::into)
}
