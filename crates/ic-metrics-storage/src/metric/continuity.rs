//! Checkpoint regression checks over canonically restored arithmetic values.
use ic_metrics::{
    MeasurementAggregate, MeasurementCounter, MeasurementHistogram, MeasurementSummary,
};

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

pub(super) const fn counter(current: &MeasurementCounter, previous: &MeasurementCounter) -> bool {
    current.total() >= previous.total()
        && (!previous.total_overflowed() || current.total_overflowed())
}

pub(super) fn aggregate(current: &MeasurementAggregate, previous: &MeasurementAggregate) -> bool {
    aggregate_change(*current, *previous).is_some()
}

pub(super) fn summary(current: &MeasurementSummary, previous: &MeasurementSummary) -> bool {
    let Some(change) = aggregate_change(current.aggregate(), previous.aggregate()) else {
        return false;
    };
    if !change.exact {
        return true;
    }
    if change.samples == 0 {
        return current.latest() == previous.latest();
    }
    let latest = u128::from(current.latest().unwrap_or(0));
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

pub(super) fn histogram<const N: usize>(
    current: &MeasurementHistogram<N>,
    previous: &MeasurementHistogram<N>,
) -> bool {
    let Some(change) = aggregate_change(current.aggregate(), previous.aggregate()) else {
        return false;
    };
    let mut maximum_added = !change.maximum_changed;
    let mut minimum_total = 0_u128;
    let mut maximum_total = 0_u128;
    let counts = histogram_counts(current).zip(histogram_counts(previous));
    let bounds = current
        .upper_bounds()
        .iter()
        .copied()
        .chain(std::iter::once(change.maximum));
    let mut lower = 0;
    for (((now, overflowed), (prior, prior_overflowed)), bound) in counts.zip(bounds) {
        if now < prior || (prior_overflowed && !overflowed) {
            return false;
        }
        let added = now - prior;
        let interval_lower = lower;
        lower = bound.saturating_add(1);
        if !change.exact || added == 0 {
            continue;
        }
        let (lower, upper) = (interval_lower, bound.min(change.maximum));
        let added = u128::from(added);
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

// Finite and overflow counts follow the same monotonic/sticky rule.
fn histogram_counts<const N: usize>(
    value: &MeasurementHistogram<N>,
) -> impl Iterator<Item = (u64, bool)> + '_ {
    value
        .bucket_counts()
        .iter()
        .copied()
        .zip(value.bucket_overflowed().iter().copied())
        .chain(std::iter::once((
            value.overflow(),
            value.overflow_count_overflowed(),
        )))
}
