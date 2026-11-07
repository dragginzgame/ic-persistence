//! Packed child-address decoding, including incomplete backing reads.
use super::*;
use std::{cell::Cell, panic::AssertUnwindSafe};

struct Input {
    bytes: Vec<u8>,
    read: Cell<Option<(u64, usize)>>,
    interrupt: bool,
}

impl Memory for Input {
    fn size(&self) -> u64 {
        unreachable!()
    }
    fn grow(&self, _: u64) -> i64 {
        unreachable!()
    }
    fn read(&self, _: u64, _: &mut [u8]) {
        unreachable!("child addresses use the initializing read hook")
    }
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        assert!(self.read.replace(Some((offset, count))).is_none());
        let offset = usize::try_from(offset).unwrap();
        let source = &self.bytes[offset..offset + count];
        let copied = if self.interrupt { count.min(1) } else { count };
        // SAFETY: caller supplies a separate destination for count bytes;
        // the complete source range is admitted before any partial copy.
        unsafe { std::ptr::copy_nonoverlapping(source.as_ptr(), dst, copied) };
        assert!(!self.interrupt, "interrupted backing read");
    }
    fn write(&self, _: u64, _: &[u8]) {
        unreachable!()
    }
}

#[test]
fn packed_child_addresses_decode_full_width_words_from_unaligned_ranges() {
    let words = [0, 1, 0x0123_4567_89ab_cdef, u64::MAX];
    for count in [0, 1, 4, 5, 11, 12, 257] {
        let expected: Vec<_> = words.into_iter().cycle().take(count).collect();
        let mut bytes = vec![0xaa; 3];
        bytes.extend(expected.iter().flat_map(|word| word.to_le_bytes()));
        bytes.push(0xbb);
        let input = Input {
            bytes,
            read: Cell::new(None),
            interrupt: false,
        };
        let actual = read_address_vec(&input, Address::from(3), count);
        assert_eq!(
            actual.into_iter().map(Address::get).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(input.read.get(), Some((3, count * 8)));
    }
}

#[test]
fn incomplete_child_reads_trap_without_publishing_addresses() {
    let input = Input {
        bytes: vec![0xff; 96],
        read: Cell::new(None),
        interrupt: true,
    };
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            read_address_vec(&input, Address::from(0), 12)
        }))
        .is_err()
    );
    assert_eq!(input.read.get(), Some((0, 96)));
    input.read.set(None);
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            read_address_vec(&input, Address::from(0), usize::MAX)
        }))
        .is_err()
    );
    assert_eq!(input.read.get(), None);
}
