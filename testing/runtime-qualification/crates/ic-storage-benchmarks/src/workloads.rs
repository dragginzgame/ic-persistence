//! Fixed consumer-shaped experiments, not production allocation presets.
use crate::{Case, Pattern};

///
/// Workload
///
/// Small repeatable selections grounded in the local consumer memory inventory.
/// Each case still owns one fresh collection; these are not full application traces.
///
#[derive(Clone, Copy, Debug, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Workload {
    Control,
    Records,
    Bulk,
}

impl Workload {
    pub fn cases(self) -> &'static [Case] {
        match self {
            Self::Control => &[Case::CellBytes, Case::U64Bytes],
            Self::Records => &[
                Case::U64Bytes,
                Case::Bytes32Bytes32,
                Case::SetBytes64,
                Case::VectorRecord,
            ],
            Self::Bulk => &[
                Case::U64Bytes,
                Case::VectorBytes,
                Case::CellBytes,
                Case::RegionBytes,
            ],
        }
    }
    pub fn payloads(self, case: Case) -> &'static [u32] {
        match (self, case) {
            (Self::Control, _) => &[256, 4096],
            (Self::Records, Case::U64Bytes) => &[31, 32, 33, 256, 1024, 4096],
            (Self::Records, _) => case.payloads(),
            (Self::Bulk, _) => &[65_536, 1_048_576],
        }
    }
    pub const fn rows(self) -> u32 {
        match self {
            Self::Control => 16,
            Self::Records => 4096,
            Self::Bulk => 8,
        }
    }
    pub const fn counts(self) -> &'static [u32] {
        match self {
            Self::Records => &[1, 32],
            Self::Control | Self::Bulk => &[1],
        }
    }
    pub const fn patterns(self) -> &'static [Pattern] {
        match self {
            Self::Records => &[Pattern::Uniform, Pattern::Hot16, Pattern::Zipf],
            Self::Control | Self::Bulk => &[Pattern::Uniform],
        }
    }
    pub const fn rounds(self) -> u32 {
        match self {
            Self::Control | Self::Records => 3,
            Self::Bulk => 1,
        }
    }
}

/// Bound setup and verification work independently of collection population.
/// A single 1 MiB value is admitted; batches otherwise target at most 512 KiB.
pub fn transfer_rows(payload: usize) -> usize {
    (512 * 1024 / payload.max(1)).clamp(1, 128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LIMITS, RUN_ID, Response, value};

    #[test]
    fn every_profile_result_fits_the_existing_wire_budget() {
        for profile in [Workload::Control, Workload::Records, Workload::Bulk] {
            for &case in profile.cases() {
                for &payload in profile.payloads(case) {
                    assert!(case.accepts_payload(payload));
                    let count = *profile.counts().last().unwrap();
                    let response = Response {
                        instructions: 0,
                        stable_pages: 0,
                        length: u64::from(count),
                        entries: (0..u64::from(count))
                            .map(|id| (id, Some(case.expected_value(id, payload, false))))
                            .collect(),
                    };
                    let encoded =
                        ic_wire::native::encode_response(RUN_ID, &response, LIMITS).unwrap();
                    let decoded: Response =
                        ic_wire::native::decode_response(RUN_ID, &encoded, LIMITS).unwrap();
                    assert_eq!(decoded.entries, response.entries);
                }
            }
        }
        let rows: Vec<_> = (0..transfer_rows(1_048_576))
            .map(|id| (id as u64, value(id as u64, 1_048_576, false)))
            .collect();
        let bytes = ic_wire::native::encode_response(3, &rows, LIMITS).unwrap();
        let decoded: Vec<(u64, Vec<u8>)> =
            ic_wire::native::decode_response(3, &bytes, LIMITS).unwrap();
        assert_eq!(rows, decoded);
    }
}
