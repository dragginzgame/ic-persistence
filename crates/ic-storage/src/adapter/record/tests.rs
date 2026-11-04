use super::*;
use crate::{VectorMemory, testing::MeteredMemory};

#[test]
fn payload_reads_admit_lengths_before_io_and_check_integrity() {
    let memory = MeteredMemory::<VectorMemory>::default();
    memory.grow(1);
    let magic = *b"PAYLOAD1";
    let payload = vec![7; 4096];
    let frame = encode_payload(magic, &payload).unwrap();
    memory.write(0, &frame);
    memory.reset();
    assert_eq!(read_payload(&memory, 0, PAGE, magic).unwrap(), payload);
    assert_eq!(memory.counts().read_bytes, frame.len() as u64);
    for (offset, replacement, reads) in [
        (0, vec![0], 1),
        (8, u32::MAX.to_le_bytes().to_vec(), 1),
        (8, 8192_u32.to_le_bytes().to_vec(), 1),
        (12, vec![0; 8], 2),
        (PREFIX as u64, vec![8], 2),
    ] {
        memory.write(0, &frame);
        memory.write(offset, &replacement);
        let before = memory.backing().borrow().clone();
        memory.reset();
        assert!(read_payload(&memory, 0, frame.len() as u64, magic).is_err());
        assert_eq!(memory.counts().reads, reads);
        assert_eq!(memory.counts().writes, 0);
        assert_eq!(*memory.backing().borrow(), before);
    }
    for (offset, capacity) in [(0, 19), (0, PAGE + 1), (u64::MAX, PAGE)] {
        memory.reset();
        assert!(read_payload(&memory, offset, capacity, magic).is_err());
        assert_eq!(memory.counts().reads, 0);
    }
}
