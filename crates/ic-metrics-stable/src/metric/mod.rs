//! Private adapters to the core's canonical restored-state admission.

use ic_metrics::{
    MeasurementAggregate, MeasurementCounter, MeasurementHistogram, MeasurementSummary,
};

use crate::{
    MAX_HISTOGRAM_BOUNDS, StoreError,
    codec::{self, Decoder, MetricKind},
};

pub trait StoredMetric: Sized + Eq {
    type Configuration: ?Sized;
    fn configuration(&self) -> &Self::Configuration;
    const KIND: MetricKind;
    fn same_state(&self, previous: &Self) -> bool {
        self == previous
    }
    const ENCODED_SIZE: Result<usize, StoreError>;
    // Called only after ENCODED_SIZE admits this metric's static shape.
    fn encode_state(&self, bytes: &mut Vec<u8>);
    fn decode_state(
        configuration: &Self::Configuration,
        decoder: &mut Decoder<'_>,
    ) -> Result<Self, StoreError>;
    fn continues(&self, previous: &Self) -> bool;
}

fn continues(current: MeasurementAggregate, previous: MeasurementAggregate) -> bool {
    if !(current.samples() >= previous.samples()
        && current.total() >= previous.total()
        && current.maximum() >= previous.maximum()
        && (!previous.samples_overflowed() || current.samples_overflowed())
        && (!previous.total_overflowed() || current.total_overflowed())
        && (current.samples_overflowed()
            || current.samples() != previous.samples()
            || current == previous))
    {
        return false;
    }
    if current.samples_overflowed() {
        return true;
    }
    // Monotonic, exact counts admit these differences. Added observations are
    // bounded by the current maximum and must attain it if that maximum grew.
    let samples = current.samples() - previous.samples();
    let total = current.total() - previous.total();
    let maximum = u128::from(current.maximum().unwrap_or(0));
    total <= u128::from(samples) * maximum
        && (current.maximum() == previous.maximum() || total >= maximum)
}

impl StoredMetric for MeasurementCounter {
    const KIND: MetricKind = MetricKind::Counter;
    type Configuration = ();

    fn configuration(&self) -> &() {
        &()
    }

    const ENCODED_SIZE: Result<usize, StoreError> = Ok(codec::COUNTER_BYTES);

    fn encode_state(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.total().to_le_bytes());
        bytes.push(u8::from(self.total_overflowed()));
    }

    fn decode_state((): &(), decoder: &mut Decoder<'_>) -> Result<Self, StoreError> {
        Ok(Self::from_parts(decoder.u128()?, decoder.flag()?)?)
    }

    fn continues(&self, previous: &Self) -> bool {
        self.total() >= previous.total()
            && (!previous.total_overflowed() || self.total_overflowed())
    }
}

impl StoredMetric for MeasurementAggregate {
    const KIND: MetricKind = MetricKind::Aggregate;
    type Configuration = ();

    fn configuration(&self) -> &() {
        &()
    }

    const ENCODED_SIZE: Result<usize, StoreError> = Ok(codec::AGGREGATE_BYTES);

    fn encode_state(&self, bytes: &mut Vec<u8>) {
        codec::aggregate(bytes, *self);
    }

    fn decode_state((): &(), decoder: &mut Decoder<'_>) -> Result<Self, StoreError> {
        decoder.aggregate()
    }

    fn continues(&self, previous: &Self) -> bool {
        continues(*self, *previous)
    }
}

impl StoredMetric for MeasurementSummary {
    const KIND: MetricKind = MetricKind::Summary;
    type Configuration = ();

    fn configuration(&self) -> &() {
        &()
    }

    const ENCODED_SIZE: Result<usize, StoreError> = Ok(codec::SUMMARY_BYTES);

    fn encode_state(&self, bytes: &mut Vec<u8>) {
        codec::aggregate(bytes, self.aggregate());
        bytes.extend_from_slice(&self.latest().unwrap_or(0).to_le_bytes());
    }

    fn decode_state((): &(), decoder: &mut Decoder<'_>) -> Result<Self, StoreError> {
        let aggregate = decoder.aggregate()?;
        let latest = decoder.u64()?;
        if aggregate.samples() == 0 && latest != 0 {
            return Err(StoreError::MalformedRecord);
        }
        Ok(Self::from_parts(
            aggregate,
            (aggregate.samples() != 0).then_some(latest),
        )?)
    }

    fn continues(&self, previous: &Self) -> bool {
        let current = self.aggregate();
        let before = previous.aggregate();
        if !continues(current, before) {
            return false;
        }
        if current.samples_overflowed() {
            return true;
        }
        let samples = current.samples() - before.samples();
        if samples == 0 {
            return self.latest() == previous.latest();
        }
        let total = current.total() - before.total();
        let latest = u128::from(self.latest().unwrap_or(0));
        let maximum = u128::from(current.maximum().unwrap_or(0));
        let other_maximum = if current.maximum() != before.maximum() && latest < maximum {
            maximum
        } else {
            0
        };
        // The last added observation is fixed. A distinct new maximum must
        // occur among the other additions; every other value is at most it.
        total >= latest + other_maximum && total <= u128::from(samples - 1) * maximum + latest
    }
}

impl<const N: usize> StoredMetric for MeasurementHistogram<N> {
    const KIND: MetricKind = MetricKind::Histogram;
    type Configuration = [u64; N];

    fn configuration(&self) -> &[u64; N] {
        self.upper_bounds()
    }

    fn same_state(&self, previous: &Self) -> bool {
        // Tiny arrays already reject cheaply. Larger matching arrays are costly
        // when recording changes a bucket but leaves overflow unchanged.
        if N < 4 {
            self == previous
        } else {
            self.overflow() == previous.overflow()
                && self.aggregate() == previous.aggregate()
                && self == previous
        }
    }

    const ENCODED_SIZE: Result<usize, StoreError> = if N > MAX_HISTOGRAM_BOUNDS {
        Err(StoreError::TooManyBounds)
    } else {
        Ok(codec::HISTOGRAM_BASE_BYTES + codec::HISTOGRAM_BUCKET_BYTES * N)
    };

    fn encode_state(&self, bytes: &mut Vec<u8>) {
        codec::aggregate(bytes, self.aggregate());
        #[expect(
            clippy::cast_possible_truncation,
            reason = "ENCODED_SIZE admits at most 256 bounds before encoding"
        )]
        let count = N as u16;
        bytes.extend_from_slice(&count.to_le_bytes());
        for ((bound, count), flag) in self
            .upper_bounds()
            .iter()
            .zip(self.bucket_counts())
            .zip(self.bucket_overflowed())
        {
            bytes.extend_from_slice(&bound.to_le_bytes());
            bytes.extend_from_slice(&count.to_le_bytes());
            bytes.push(u8::from(*flag));
        }
        bytes.extend_from_slice(&self.overflow().to_le_bytes());
        bytes.push(u8::from(self.overflow_count_overflowed()));
    }

    fn decode_state(
        configuration: &Self::Configuration,
        decoder: &mut Decoder<'_>,
    ) -> Result<Self, StoreError> {
        let aggregate = decoder.aggregate()?;
        let length = usize::from(decoder.u16()?);
        if length > MAX_HISTOGRAM_BOUNDS {
            return Err(StoreError::TooManyBounds);
        }
        if length != N {
            return Err(StoreError::BoundsMismatch);
        }
        let mut bounds = [0; N];
        let mut counts = [0; N];
        let mut flags = [false; N];
        for ((bound, count), flag) in bounds.iter_mut().zip(&mut counts).zip(&mut flags) {
            *bound = decoder.u64()?;
            *count = decoder.u64()?;
            *flag = decoder.flag()?;
        }
        let overflow = decoder.u64()?;
        let overflow_count_overflowed = decoder.flag()?;
        let value = Self::from_parts(
            bounds,
            counts,
            overflow,
            flags,
            overflow_count_overflowed,
            aggregate,
        )?;
        if value.upper_bounds() != configuration {
            return Err(StoreError::BoundsMismatch);
        }
        Ok(value)
    }

    fn continues(&self, previous: &Self) -> bool {
        let current = self.aggregate();
        let before = previous.aggregate();
        if !continues(current, before) {
            return false;
        }
        let exact = !current.samples_overflowed();
        let maximum = current.maximum().unwrap_or(0);
        let maximum_changed = current.maximum() != before.maximum();
        let mut maximum_added = !maximum_changed;
        let mut minimum_total = 0_u128;
        let mut maximum_total = 0_u128;
        for index in 0..=N {
            let (now, prior, overflowed, prior_overflowed) = if index < N {
                (
                    self.bucket_counts()[index],
                    previous.bucket_counts()[index],
                    self.bucket_overflowed()[index],
                    previous.bucket_overflowed()[index],
                )
            } else {
                (
                    self.overflow(),
                    previous.overflow(),
                    self.overflow_count_overflowed(),
                    previous.overflow_count_overflowed(),
                )
            };
            if now < prior || (prior_overflowed && !overflowed) {
                return false;
            }
            if !exact || now == prior {
                continue;
            }
            let lower = if index == 0 {
                0
            } else {
                self.upper_bounds()[index - 1].saturating_add(1)
            };
            let upper = if index < N {
                self.upper_bounds()[index].min(maximum)
            } else {
                maximum
            };
            let added = u128::from(now - prior);
            // Exact aggregate counts bound the sum of additions by u64::MAX,
            // so both range totals, including attainment, fit in u128.
            minimum_total += added * u128::from(lower);
            maximum_total += added * u128::from(upper);
            if maximum_changed && upper == maximum {
                minimum_total += u128::from(maximum - lower);
                maximum_added = true;
            }
        }
        !exact
            || (maximum_added
                && (minimum_total..=maximum_total).contains(&(current.total() - before.total())))
    }
}
