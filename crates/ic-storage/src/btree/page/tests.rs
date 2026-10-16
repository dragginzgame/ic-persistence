use super::{Error, SIZE, checksum, number, seal, verify};

#[test]
fn checksum_preserves_persisted_fnv1a_bytes() {
    let mut state = 0x6a09_e667_f3bc_c909u64;
    for pattern in 0..66 {
        let mut page = [0; SIZE];
        for byte in &mut page {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = match pattern {
                0 => 0,
                1 => 255,
                _ => state.to_le_bytes()[0],
            };
        }
        // The original persisted algorithm, independently of chunk boundaries.
        let expected = page[..SIZE - 8]
            .iter()
            .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
            });
        assert_eq!(checksum(&page), expected);
        seal(&mut page);
        assert_eq!(number(&page, SIZE - 8), expected);
        assert_eq!(verify(&page), Ok(()));
    }
}

#[test]
fn checksum_detects_corruption_at_every_page_byte() {
    let mut page = [0x5a; SIZE];
    seal(&mut page);
    for at in 0..SIZE {
        page[at] ^= 1;
        assert_eq!(verify(&page), Err(Error::Corrupt), "byte {at}");
        page[at] ^= 1;
    }
    assert_eq!(verify(&page), Ok(()));
}

#[test]
fn free_pages_preserve_bytes_and_validate_padding() {
    for next in (0..64)
        .flat_map(|bit| [1u64 << bit, !(1u64 << bit)])
        .chain([0, u64::MAX])
    {
        let mut expected = [0; SIZE];
        expected[0] = super::FREE;
        super::put(&mut expected, 8, next);
        seal(&mut expected);
        let actual = super::free(next);
        assert_eq!(*actual, expected, "free-list successor {next}");
        assert_eq!(verify(&actual), Ok(()));
    }
    let mut page = super::free(42);
    for at in 0..SIZE {
        page[at] ^= 1;
        assert_eq!(verify(&page), Err(Error::Corrupt), "free-page byte {at}");
        page[at] ^= 1;
    }
}

#[test]
fn zero_suffix_checksum_matches_original_at_every_boundary() {
    let mut page = [0; SIZE];
    for at in 0..SIZE - 8 {
        page[at] = 0xa5;
        let expected = page[..SIZE - 8]
            .iter()
            .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
            });
        assert_eq!(checksum(&page), expected, "last nonzero byte {at}");
        page[at] = 0;
    }
}

#[test]
fn heap_blob_encoding_preserves_every_page_byte() {
    for len in 0..=super::PAYLOAD {
        for next in [0, 1, u64::MAX] {
            let payload = (0..len).map(|at| at.to_le_bytes()[0]).collect::<Vec<_>>();
            let mut expected = [0; SIZE];
            expected[0] = super::BLOB;
            super::put(&mut expected, 8, next);
            expected[16..16 + len].copy_from_slice(&payload);
            seal(&mut expected);
            let actual = super::encode_blob(next, &payload);
            assert_eq!(*actual, expected);
            assert_eq!(verify(&actual), Ok(()));
        }
    }
}
