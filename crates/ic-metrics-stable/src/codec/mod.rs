//! The single bounded V1 stored-record encoder and decoder.
//!
//! Store entrypoints admit the static metric size before codec dispatch.
//! This module admits stored bytes, identities and numeric state.

use ic_metrics::MeasurementAggregate;

use crate::{MAX_HISTOGRAM_BOUNDS, MAX_UNIT_BYTES, MetricIdentity, StoreError};

pub const AGGREGATE_BYTES: usize = 33;
pub const COUNTER_BYTES: usize = 16 + 1;
pub const SUMMARY_BYTES: usize = AGGREGATE_BYTES + 8;
pub const HISTOGRAM_BASE_BYTES: usize = AGGREGATE_BYTES + 2 + 9;
pub const HISTOGRAM_BUCKET_BYTES: usize = 17;

const MAGIC: &[u8; 4] = b"ICM\x01";
const MAX_RECORD_BYTES: usize = 4608;
const HEADER_BYTES: usize = 4 + 1 + 1 + 8;

/// The frozen V1 discriminators; conversion admits unknown stored bytes here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MetricKind {
    Aggregate = 1,
    Summary = 2,
    Histogram = 3,
    Counter = 4,
}

impl MetricKind {
    const fn from_byte(byte: u8) -> Result<Self, StoreError> {
        match byte {
            value if value == Self::Aggregate as u8 => Ok(Self::Aggregate),
            value if value == Self::Summary as u8 => Ok(Self::Summary),
            value if value == Self::Histogram as u8 => Ok(Self::Histogram),
            value if value == Self::Counter as u8 => Ok(Self::Counter),
            _ => Err(StoreError::KindMismatch),
        }
    }
}

// Only the sealed, shape-admitted encoders can write records. Keep their budgets
// tied to the same constants used by identity and payload-size admission.
const _: () = {
    assert!(MAX_UNIT_BYTES <= u8::MAX as usize);
    assert!(HEADER_BYTES + MAX_UNIT_BYTES + AGGREGATE_BYTES <= MAX_RECORD_BYTES);
    assert!(HEADER_BYTES + MAX_UNIT_BYTES + COUNTER_BYTES <= MAX_RECORD_BYTES);
    assert!(HEADER_BYTES + MAX_UNIT_BYTES + SUMMARY_BYTES <= MAX_RECORD_BYTES);
    assert!(
        HEADER_BYTES
            + MAX_UNIT_BYTES
            + HISTOGRAM_BASE_BYTES
            + HISTOGRAM_BUCKET_BYTES * MAX_HISTOGRAM_BOUNDS
            <= MAX_RECORD_BYTES
    );
};

pub struct Decoder<'a> {
    remaining: &'a [u8],
}

impl<'a> Decoder<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], StoreError> {
        let (bytes, remaining) = self
            .remaining
            .split_at_checked(length)
            .ok_or(StoreError::MalformedRecord)?;
        self.remaining = remaining;
        Ok(bytes)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], StoreError> {
        let (bytes, remaining) = self
            .remaining
            .split_first_chunk::<N>()
            .ok_or(StoreError::MalformedRecord)?;
        self.remaining = remaining;
        Ok(*bytes)
    }

    pub fn byte(&mut self) -> Result<u8, StoreError> {
        Ok(u8::from_le_bytes(self.array()?))
    }

    pub fn flag(&mut self) -> Result<bool, StoreError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(StoreError::MalformedRecord),
        }
    }

    pub fn u16(&mut self) -> Result<u16, StoreError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub fn u64(&mut self) -> Result<u64, StoreError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    pub fn u128(&mut self) -> Result<u128, StoreError> {
        Ok(u128::from_le_bytes(self.array()?))
    }

    pub fn aggregate(&mut self) -> Result<MeasurementAggregate, StoreError> {
        let samples = self.u64()?;
        let total = self.u128()?;
        let maximum = self.u64()?;
        let flags = self.byte()?;
        if flags > 3 || (samples == 0 && maximum != 0) {
            return Err(StoreError::MalformedRecord);
        }
        Ok(MeasurementAggregate::from_parts(
            samples,
            total,
            (samples != 0).then_some(maximum),
            flags & 1 != 0,
            flags & 2 != 0,
        )?)
    }
}

pub fn aggregate(bytes: &mut Vec<u8>, value: MeasurementAggregate) {
    bytes.extend_from_slice(&value.samples().to_le_bytes());
    bytes.extend_from_slice(&value.total().to_le_bytes());
    bytes.extend_from_slice(&value.maximum().unwrap_or(0).to_le_bytes());
    bytes.push(u8::from(value.samples_overflowed()) | (u8::from(value.total_overflowed()) << 1));
}

pub fn envelope<'a>(
    bytes: &'a [u8],
    identity: &MetricIdentity,
) -> Result<(MetricKind, Decoder<'a>), StoreError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(StoreError::RecordTooLarge);
    }
    let mut decoder = Decoder { remaining: bytes };
    if decoder.take(4)? != MAGIC {
        return Err(StoreError::UnsupportedFormat);
    }
    let kind = MetricKind::from_byte(decoder.byte()?)?;
    let length = usize::from(decoder.byte()?);
    let unit = decoder.take(length)?;
    crate::identifier(unit, MAX_UNIT_BYTES).map_err(|_| StoreError::MalformedRecord)?;
    let epoch = decoder.u64()?;
    if unit != identity.unit.as_bytes() || epoch != identity.epoch {
        return Err(StoreError::IdentityMismatch);
    }
    Ok((kind, decoder))
}

pub fn encode<T: crate::metric::StoredMetric>(
    identity: &MetricIdentity,
    value: &T,
    payload_size: usize,
) -> Vec<u8> {
    // The store has already admitted T::ENCODED_SIZE before reading any row.
    // Private encoders write that exact payload size; units are constructor-admitted.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "constructor-admitted units fit in u8 by the layout budget assertion"
    )]
    let length = identity.unit.len() as u8;
    let capacity = HEADER_BYTES + identity.unit.len() + payload_size;
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&[T::KIND as u8, length]);
    bytes.extend_from_slice(identity.unit.as_bytes());
    bytes.extend_from_slice(&identity.epoch.to_le_bytes());
    value.encode_state(&mut bytes);
    bytes
}

pub fn decode<T: crate::metric::StoredMetric>(
    bytes: &[u8],
    identity: &MetricIdentity,
    configuration: &T::Configuration,
) -> Result<T, StoreError> {
    let (kind, mut decoder) = envelope(bytes, identity)?;
    if kind != T::KIND {
        return Err(StoreError::KindMismatch);
    }
    let value = T::decode_state(configuration, &mut decoder)?;
    if !decoder.remaining.is_empty() {
        return Err(StoreError::MalformedRecord);
    }
    Ok(value)
}
