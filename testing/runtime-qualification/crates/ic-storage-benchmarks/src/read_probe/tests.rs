use super::*;

#[test]
fn largest_combined_response_fits_the_shared_probe_limits() {
    let reads = vec![
        Read {
            phase: "combined".into(),
            offset: 4090,
            destination: 0,
            instructions: 5000,
            bytes: vec![7; 4096 + 12],
        },
        Read {
            phase: "repeat".into(),
            offset: 4102,
            destination: 8192,
            instructions: 85000,
            bytes: vec![8; 4096],
        },
    ];
    let bytes = ic_wire::native::encode_response(ID, &reads, LIMITS).unwrap();
    let decoded: Vec<Read> = ic_wire::native::decode_response(ID, &bytes, LIMITS).unwrap();
    assert_eq!(decoded.len(), reads.len());
    for (a, b) in reads.into_iter().zip(decoded) {
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(
            (a.offset, a.destination, a.instructions),
            (b.offset, b.destination, b.instructions)
        );
    }
}
