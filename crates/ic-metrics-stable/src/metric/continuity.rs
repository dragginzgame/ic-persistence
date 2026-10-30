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
    for index in 0..=N {
        let Some(added) = histogram_bucket_additions(current, previous, index) else {
            return false;
        };
        if !change.exact || added == 0 {
            continue;
        }
        let (lower, upper) = histogram_interval(current, index, change.maximum);
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

// Project the finite buckets and the overflow bucket under the same sticky-count rule.
fn histogram_bucket_additions<const N: usize>(
    current: &MeasurementHistogram<N>,
    previous: &MeasurementHistogram<N>,
    index: usize,
) -> Option<u64> {
    let (now, prior, overflowed, prior_overflowed) = if index < N {
        (
            current.bucket_counts()[index],
            previous.bucket_counts()[index],
            current.bucket_overflowed()[index],
            previous.bucket_overflowed()[index],
        )
    } else {
        (
            current.overflow(),
            previous.overflow(),
            current.overflow_count_overflowed(),
            previous.overflow_count_overflowed(),
        )
    };
    if now < prior || (prior_overflowed && !overflowed) {
        None
    } else {
        Some(now - prior)
    }
}

// Called only for exact, changed counts; unchanged/saturated buckets stay cheap.
fn histogram_interval<const N: usize>(
    current: &MeasurementHistogram<N>,
    index: usize,
    maximum: u64,
) -> (u64, u64) {
    let lower = if index == 0 {
        0
    } else {
        current.upper_bounds()[index - 1].saturating_add(1)
    };
    let upper = if index < N {
        current.upper_bounds()[index].min(maximum)
    } else {
        maximum
    };
    (lower, upper)
}
