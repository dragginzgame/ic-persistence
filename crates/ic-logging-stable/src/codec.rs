//! Bounded V1 bytes; all decoded rows enter Logging's canonical restore boundary.

use std::num::NonZeroUsize;

use ic_logging::{Entry, Level, Limits, Log, LogSnapshot};

use crate::{LogIdentity, MAX_RECORD_BYTES, StoreError};

const MAGIC: &[u8] = b"ICL\x01";

pub fn encode(identity: LogIdentity, state: &LogSnapshot) -> Result<Vec<u8>, StoreError> {
    let mut writer = Writer(Vec::new());
    writer.put(MAGIC)?;
    writer.put(&identity.epoch.to_le_bytes())?;
    for value in [
        state.limits.max_entries.get(),
        state.limits.max_message_bytes,
        state.limits.max_label_bytes,
    ] {
        writer.size(value)?;
    }
    writer.option(state.next_sequence)?;
    writer.option(state.last_created_at_secs)?;
    writer.size(state.entries.len())?;
    for entry in &state.entries {
        writer.put(&entry.sequence.to_le_bytes())?;
        writer.put(&entry.created_at_secs.to_le_bytes())?;
        writer.put(&[level_tag(entry.level), u8::from(entry.truncated)])?;
        writer.text(&entry.source)?;
        writer.put(&[u8::from(entry.topic.is_some())])?;
        if let Some(topic) = &entry.topic {
            writer.text(topic)?;
        }
        writer.text(&entry.message)?;
    }
    Ok(writer.0)
}

pub fn decode(bytes: &[u8], identity: LogIdentity, limits: Limits) -> Result<Log, StoreError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(StoreError::RecordTooLarge);
    }
    let mut reader = Reader(bytes);
    if reader.take(4)? != MAGIC {
        return Err(StoreError::UnsupportedFormat);
    }
    if reader.number()? != identity.epoch {
        return Err(StoreError::IdentityMismatch);
    }
    let stored_limits = Limits {
        max_entries: NonZeroUsize::new(reader.size()?).ok_or(StoreError::MalformedRecord)?,
        max_message_bytes: reader.size()?,
        max_label_bytes: reader.size()?,
    };
    if stored_limits != limits {
        return Err(StoreError::LimitsMismatch);
    }
    let next_sequence = reader.option()?;
    let last_created_at_secs = reader.option()?;
    let count = reader.size()?;
    if count > limits.max_entries.get() {
        return Err(StoreError::InvalidState(
            ic_logging::RestoreError::TooManyEntries,
        ));
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let sequence = reader.number()?;
        let created_at_secs = reader.number()?;
        let level = read_level(reader.byte()?)?;
        let truncated = reader.flag()?;
        let source = reader.text(limits.max_label_bytes)?;
        let topic = if reader.flag()? {
            Some(reader.text(limits.max_label_bytes)?)
        } else {
            None
        };
        let message = reader.text(limits.max_message_bytes)?;
        entries.push(Entry {
            sequence,
            created_at_secs,
            level,
            source,
            topic,
            message,
            truncated,
        });
    }
    if !reader.0.is_empty() {
        return Err(StoreError::MalformedRecord);
    }
    Log::restore(LogSnapshot {
        limits,
        entries,
        next_sequence,
        last_created_at_secs,
    })
    .map_err(Into::into)
}

struct Writer(Vec<u8>);

impl Writer {
    fn put(&mut self, bytes: &[u8]) -> Result<(), StoreError> {
        if bytes.len() > MAX_RECORD_BYTES - self.0.len() {
            return Err(StoreError::RecordTooLarge);
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn size(&mut self, value: usize) -> Result<(), StoreError> {
        self.put(
            &u32::try_from(value)
                .map_err(|_| StoreError::RecordTooLarge)?
                .to_le_bytes(),
        )
    }
    fn option(&mut self, value: Option<u64>) -> Result<(), StoreError> {
        self.put(&[u8::from(value.is_some())])?;
        if let Some(value) = value {
            self.put(&value.to_le_bytes())?;
        }
        Ok(())
    }
    fn text(&mut self, value: &str) -> Result<(), StoreError> {
        self.size(value.len())?;
        self.put(value.as_bytes())
    }
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], StoreError> {
        let (head, tail) = self
            .0
            .split_at_checked(len)
            .ok_or(StoreError::MalformedRecord)?;
        self.0 = tail;
        Ok(head)
    }
    fn byte(&mut self) -> Result<u8, StoreError> {
        self.take(1)?
            .first()
            .copied()
            .ok_or(StoreError::MalformedRecord)
    }
    fn flag(&mut self) -> Result<bool, StoreError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(StoreError::MalformedRecord),
        }
    }
    fn number(&mut self) -> Result<u64, StoreError> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| StoreError::MalformedRecord)?,
        ))
    }
    fn size(&mut self) -> Result<usize, StoreError> {
        let number = u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| StoreError::MalformedRecord)?,
        );
        usize::try_from(number).map_err(|_| StoreError::MalformedRecord)
    }
    fn option(&mut self) -> Result<Option<u64>, StoreError> {
        if self.flag()? {
            self.number().map(Some)
        } else {
            Ok(None)
        }
    }
    fn text(&mut self, maximum: usize) -> Result<String, StoreError> {
        let size = self.size()?;
        if size > maximum {
            return Err(StoreError::MalformedRecord);
        }
        std::str::from_utf8(self.take(size)?)
            .map(String::from)
            .map_err(|_| StoreError::MalformedRecord)
    }
}

const fn level_tag(level: Level) -> u8 {
    match level {
        Level::Debug => 0,
        Level::Info => 1,
        Level::Ok => 2,
        Level::Warn => 3,
        Level::Error => 4,
    }
}

const fn read_level(tag: u8) -> Result<Level, StoreError> {
    match tag {
        0 => Ok(Level::Debug),
        1 => Ok(Level::Info),
        2 => Ok(Level::Ok),
        3 => Ok(Level::Warn),
        4 => Ok(Level::Error),
        _ => Err(StoreError::MalformedRecord),
    }
}
