use super::*;
#[test]
fn largest_matrix_responses_fit_protocol_budgets() {
    let rows: Vec<_> = (0..128).map(|id| (id, value(id, 4096, false))).collect();
    let encoded = ic_wire::native::encode_response(3, &rows, LIMITS).unwrap();
    let decoded: Vec<(u64, Vec<u8>)> =
        ic_wire::native::decode_response(3, &encoded, LIMITS).unwrap();
    assert_eq!(decoded, rows);
    let response = Response {
        instructions: 0,
        stable_pages: 0,
        length: 1,
        entries: (0..32)
            .map(|id| (id, Some(value(id, 16_384, false))))
            .collect(),
    };
    let encoded = ic_wire::native::encode_response(RUN_ID, &response, LIMITS).unwrap();
    let decoded: Response = ic_wire::native::decode_response(RUN_ID, &encoded, LIMITS).unwrap();
    assert_eq!(decoded.entries, response.entries);
}
#[test]
fn traces_are_bounded_repeatable_and_missing_keys_are_disjoint() {
    for pattern in [Pattern::Uniform, Pattern::Hot16, Pattern::Zipf] {
        for operation in OPERATIONS {
            let r = Request {
                case: Case::U64Bytes,
                operation,
                rows: 101,
                payload: 1024,
                count: 32,
                seed: 7,
                pattern,
            };
            let a = keys(r);
            assert_eq!(a, keys(r));
            assert_eq!(a.len(), 32);
            assert!(a.iter().all(|&key| {
                if matches!(
                    operation,
                    Operation::MissAbove
                        | Operation::Insert
                        | Operation::RemoveMiss
                        | Operation::RemoveInserted
                        | Operation::Push
                ) {
                    key >= 101
                } else {
                    key < if pattern == Pattern::Hot16 { 16 } else { 101 }
                }
            }));
            let bytes = ic_wire::native::encode_request(RUN_ID, &r, LIMITS).unwrap();
            let decoded: Request = ic_wire::native::decode_request(RUN_ID, &bytes, LIMITS).unwrap();
            assert_eq!(keys(decoded), a);
        }
    }
    assert_ne!(value(7, 32, false), value(7, 32, true));
}
