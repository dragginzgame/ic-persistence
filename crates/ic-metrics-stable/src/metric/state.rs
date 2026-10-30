//! Durable state schemas and canonical restoration, without continuity policy.
use crate::{MAX_HISTOGRAM_BOUNDS, StoreError};
use ic_metrics::MeasurementAggregate;
use serde::{Deserialize, Serialize};

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
    pub(super) fn restore(self) -> Result<MeasurementAggregate, StoreError> {
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
