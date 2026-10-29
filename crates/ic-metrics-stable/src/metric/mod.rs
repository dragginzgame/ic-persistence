//! Private adapters to the core's canonical restored-state admission.

use ic_metrics::{
    MeasurementAggregate, MeasurementCounter, MeasurementHistogram, MeasurementSummary,
};

use serde::{Deserialize, Serialize};

use crate::{MAX_HISTOGRAM_BOUNDS, StoreError, codec::MetricKind};

pub trait StoredMetric: Sized + Eq {
    type Configuration: ?Sized;
    fn configuration(&self) -> &Self::Configuration;
    const KIND: MetricKind;
    fn same_state(&self, previous: &Self) -> bool {
        self == previous
    }
    type State: Serialize + serde::de::DeserializeOwned;
    const ADMISSION: Result<(), StoreError> = Ok(());
    fn state(&self) -> Self::State;
    fn restore(configuration: &Self::Configuration, state: Self::State)
    -> Result<Self, StoreError>;
    fn continues(&self, previous: &Self) -> bool;
}

// These private records preserve typed arithmetic admission after Wire has
// admitted syntax and traversal. They are not a second arithmetic authority.
///
/// `AggregateState`
///
/// Stored fields admitted by the canonical arithmetic constructor after decoding.
///

#[derive(Deserialize, Serialize)]
pub struct AggregateState {
    pub samples: u64,
    pub total: u128,
    pub maximum: Option<u64>,
    pub samples_overflowed: bool,
    pub total_overflowed: bool,
}

impl From<MeasurementAggregate> for AggregateState {
    fn from(value: MeasurementAggregate) -> Self {
        Self {
            samples: value.samples(),
            total: value.total(),
            maximum: value.maximum(),
            samples_overflowed: value.samples_overflowed(),
            total_overflowed: value.total_overflowed(),
        }
    }
}

impl AggregateState {
    fn restore(self) -> Result<MeasurementAggregate, StoreError> {
        Ok(MeasurementAggregate::from_parts(
            self.samples,
            self.total,
            self.maximum,
            self.samples_overflowed,
            self.total_overflowed,
        )?)
    }
}

///
/// `HistogramState`
///
/// Bounded disjoint buckets and aggregate fields awaiting arithmetic admission.
///

#[derive(Deserialize, Serialize)]
pub struct HistogramState {
    pub aggregate: AggregateState,
    pub buckets: ic_wire::List<(u64, u64, bool), MAX_HISTOGRAM_BOUNDS>,
    pub overflow: u64,
    pub overflowed: bool,
}

// Monotonic differences are admitted once. Counts describe exact additions
// only while the aggregate sample count has not saturated.
struct AggregateChange {
    exact: bool,
    samples: u64,
    total: u128,
    maximum: u64,
    maximum_changed: bool,
}

#[inline]
fn aggregate_change(
    current: MeasurementAggregate,
    previous: MeasurementAggregate,
) -> Option<AggregateChange> {
    if !(current.samples() >= previous.samples()
        && current.total() >= previous.total()
        && current.maximum() >= previous.maximum()
        && (!previous.samples_overflowed() || current.samples_overflowed())
        && (!previous.total_overflowed() || current.total_overflowed())
        && (current.samples_overflowed()
            || current.samples() != previous.samples()
            || current == previous))
    {
        return None;
    }
    let change = AggregateChange {
        exact: !current.samples_overflowed(),
        samples: current.samples() - previous.samples(),
        total: current.total() - previous.total(),
        maximum: current.maximum().unwrap_or(0),
        maximum_changed: current.maximum() != previous.maximum(),
    };
    // Exact additions are bounded by the current maximum and must attain it
    // if that maximum grew. Saturated sample counts retain monotonic checks.
    (!change.exact
        || (change.total <= u128::from(change.samples) * u128::from(change.maximum)
            && (!change.maximum_changed || change.total >= u128::from(change.maximum))))
    .then_some(change)
}

impl StoredMetric for MeasurementCounter {
    const KIND: MetricKind = MetricKind::Counter;
    type Configuration = ();

    fn configuration(&self) -> &() {
        &()
    }

    type State = (u128, bool);

    fn state(&self) -> Self::State {
        (self.total(), self.total_overflowed())
    }

    fn restore((): &(), (total, overflowed): Self::State) -> Result<Self, StoreError> {
        Ok(Self::from_parts(total, overflowed)?)
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

    type State = AggregateState;

    fn state(&self) -> Self::State {
        (*self).into()
    }

    fn restore((): &(), state: Self::State) -> Result<Self, StoreError> {
        state.restore()
    }

    fn continues(&self, previous: &Self) -> bool {
        aggregate_change(*self, *previous).is_some()
    }
}

impl StoredMetric for MeasurementSummary {
    const KIND: MetricKind = MetricKind::Summary;
    type Configuration = ();

    fn configuration(&self) -> &() {
        &()
    }

    type State = (AggregateState, Option<u64>);

    fn state(&self) -> Self::State {
        (self.aggregate().into(), self.latest())
    }

    fn restore((): &(), (aggregate, latest): Self::State) -> Result<Self, StoreError> {
        Ok(Self::from_parts(aggregate.restore()?, latest)?)
    }

    fn continues(&self, previous: &Self) -> bool {
        let Some(change) = aggregate_change(self.aggregate(), previous.aggregate()) else {
            return false;
        };
        if !change.exact {
            return true;
        }
        if change.samples == 0 {
            return self.latest() == previous.latest();
        }
        let latest = u128::from(self.latest().unwrap_or(0));
        let maximum = u128::from(change.maximum);
        let other_maximum = if change.maximum_changed && latest < maximum {
            maximum
        } else {
            0
        };
        // The last added observation is fixed. A distinct new maximum must
        // occur among the other additions; every other value is at most it.
        change.total >= latest + other_maximum
            && change.total <= u128::from(change.samples - 1) * maximum + latest
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

    type State = HistogramState;
    const ADMISSION: Result<(), StoreError> = if N > MAX_HISTOGRAM_BOUNDS {
        Err(StoreError::TooManyBounds)
    } else {
        Ok(())
    };

    fn state(&self) -> Self::State {
        HistogramState {
            aggregate: self.aggregate().into(),
            buckets: ic_wire::List::new(
                self.upper_bounds()
                    .iter()
                    .copied()
                    .zip(self.bucket_counts().iter().copied())
                    .zip(self.bucket_overflowed().iter().copied())
                    .map(|((bound, count), flag)| (bound, count, flag))
                    .collect(),
            )
            .expect("static histogram shape was admitted before serialization"),
            overflow: self.overflow(),
            overflowed: self.overflow_count_overflowed(),
        }
    }

    fn restore(
        configuration: &Self::Configuration,
        state: Self::State,
    ) -> Result<Self, StoreError> {
        if state.buckets.as_slice().len() != N {
            return Err(StoreError::BoundsMismatch);
        }
        let mut bounds = [0; N];
        let mut counts = [0; N];
        let mut flags = [false; N];
        for (index, &(bound, count, flag)) in state.buckets.as_slice().iter().enumerate() {
            bounds[index] = bound;
            counts[index] = count;
            flags[index] = flag;
        }
        let value = Self::from_parts(
            bounds,
            counts,
            state.overflow,
            flags,
            state.overflowed,
            state.aggregate.restore()?,
        )?;
        if value.upper_bounds() != configuration {
            return Err(StoreError::BoundsMismatch);
        }
        Ok(value)
    }

    fn continues(&self, previous: &Self) -> bool {
        let Some(change) = aggregate_change(self.aggregate(), previous.aggregate()) else {
            return false;
        };
        let mut maximum_added = !change.maximum_changed;
        let mut minimum_total = 0_u128;
        let mut maximum_total = 0_u128;
        // Project finite/overflow bucket metadata separately from admission.
        // Intervals stay lazy: saturated or unchanged counts need no range math.
        let bucket = |index: usize| {
            if index < N {
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
            }
        };
        let interval = |index: usize| {
            let lower = if index == 0 {
                0
            } else {
                self.upper_bounds()[index - 1].saturating_add(1)
            };
            let upper = if index < N {
                self.upper_bounds()[index].min(change.maximum)
            } else {
                change.maximum
            };
            (lower, upper)
        };
        for index in 0..=N {
            let (now, prior, overflowed, prior_overflowed) = bucket(index);
            if now < prior || (prior_overflowed && !overflowed) {
                return false;
            }
            if !change.exact || now == prior {
                continue;
            }
            let (lower, upper) = interval(index);
            let added = u128::from(now - prior);
            // Exact aggregate counts bound the sum of additions by u64::MAX,
            // so both range totals, including attainment, fit in u128.
            minimum_total += added * u128::from(lower);
            maximum_total += added * u128::from(upper);
            if change.maximum_changed && upper == change.maximum {
                minimum_total += u128::from(change.maximum - lower);
                maximum_added = true;
            }
        }
        !change.exact || (maximum_added && (minimum_total..=maximum_total).contains(&change.total))
    }
}
