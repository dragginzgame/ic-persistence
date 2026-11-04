use super::*;
use crate::{LIMITS, Operation, RUN_ID, Request, Response, payload_for, value};

#[test]
fn every_profile_result_fits_the_existing_wire_budget() {
    for profile in [
        Workload::Control,
        Workload::Records,
        Workload::Bulk,
        Workload::Boundaries,
    ] {
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
                let encoded = ic_wire::native::encode_response(RUN_ID, &response, LIMITS).unwrap();
                let decoded: Response =
                    ic_wire::native::decode_response(RUN_ID, &encoded, LIMITS).unwrap();
                assert_eq!(decoded.entries, response.entries);
                if case.operations().contains(&Operation::Grow) {
                    let grown = payload_for(Request {
                        case,
                        operation: Operation::Grow,
                        rows: profile.rows(),
                        payload,
                        count,
                        seed: 1,
                        pattern: Pattern::Uniform,
                    });
                    assert!(grown > payload, "{case:?}: {payload} -> {grown}");
                    let response = Response {
                        entries: (0..u64::from(count))
                            .map(|id| (id, Some(case.expected_value(id, grown, true))))
                            .collect(),
                        ..response
                    };
                    let encoded = ic_wire::native::encode_response(RUN_ID, &response, LIMITS)
                        .unwrap_or_else(|e| panic!("{case:?}: {payload} -> {grown}: {e:?}"));
                    assert!(encoded.len() <= 2 * 1024 * 1024, "IC reply ceiling");
                    let decoded: Response =
                        ic_wire::native::decode_response(RUN_ID, &encoded, LIMITS).unwrap();
                    assert_eq!(decoded.entries, response.entries);
                }
            }
        }
    }
    let rows: Vec<_> = (0..transfer_rows(1_572_864))
        .map(|id| (id as u64, value(id as u64, 1_572_864, false)))
        .collect();
    let bytes = ic_wire::native::encode_response(3, &rows, LIMITS).unwrap();
    assert!(bytes.len() <= 2 * 1024 * 1024, "IC reply ceiling");
    let decoded: Vec<(u64, Vec<u8>)> = ic_wire::native::decode_response(3, &bytes, LIMITS).unwrap();
    assert_eq!(rows, decoded);
}
