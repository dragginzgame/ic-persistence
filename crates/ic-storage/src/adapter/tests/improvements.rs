//! Baseline ordered projections and caller-codec behavior.
use super::*;
use std::{
    borrow::Cow,
    panic::{AssertUnwindSafe, catch_unwind},
};
#[test]
fn bounded_map_keys_and_mixed_end_ranges_preserve_order() {
    let runtime = runtime(VectorMemory::default());
    let mut map = StableBTreeMap::<u64, Vec<u8>>::create(
        runtime
            .open_memory(&crate::StableKey::parse("test.map.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    for key in 0..100 {
        map.insert(key, vec![7; 16384]).unwrap();
    }
    let mut keys = map.keys_range(10..14);
    assert_eq!(keys.next().unwrap().unwrap(), 10);
    assert_eq!(keys.next_back().unwrap().unwrap(), 13);
    assert_eq!(keys.collect::<Result<Vec<_>, _>>().unwrap(), vec![11, 12]);
}
#[derive(Clone)]
struct RejectDecode(u64);
impl Storable for RejectDecode {
    const BOUND: Bound = u64::BOUND;
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        self.0.to_bytes()
    }
    fn from_bytes(_: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        Err(CodecError::Invalid("injected stored decode refusal"))
    }
}
#[test]
fn stored_value_decode_failure_traps_and_keys_remain_readable() {
    let backing = VectorMemory::default();
    let runtime = runtime(backing.clone());
    let mut map = StableBTreeMap::<u64, RejectDecode>::create(
        runtime
            .open_memory(&crate::StableKey::parse("test.map.v1").unwrap())
            .unwrap(),
    )
    .unwrap();
    map.insert(7, RejectDecode(42)).unwrap();
    let before = backing.borrow().clone();
    assert!(catch_unwind(AssertUnwindSafe(|| map.get(&7))).is_err());
    assert_eq!(map.keys().collect::<Result<Vec<_>, _>>().unwrap(), vec![7]);
    assert_eq!(*backing.borrow(), before);
    // Upstream updates before decoding a displaced value. A native panic must
    // poison the shared owner; IC executions use rollback instead.
    assert!(catch_unwind(AssertUnwindSafe(|| map.insert(7, RejectDecode(43)))).is_err());
    assert!(
        runtime
            .open_memory(&crate::StableKey::parse("test.set.v1").unwrap())
            .is_err()
    );
}
