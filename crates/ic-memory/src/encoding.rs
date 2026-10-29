//! Wire owns serialization and resource admission; Memory owns ledger semantics.

use serde::{Deserialize, Deserializer};

// Conservative traversal allowances, not heap measurements. The physical record
// contains opaque bytes; logical rows borrow text until their bounds are checked.
pub const RECORD_MAGIC: &[u8] = b"ICML";
pub const RECORD_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: crate::constants::MAX_LEDGER_RECORD_BYTES - RECORD_MAGIC.len(),
    allocation: 4 * 1024 * 1024,
    nodes: 16 * 1024,
    depth: crate::constants::MAX_LEDGER_NESTING,
};
pub const LEDGER_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: crate::constants::MAX_LEDGER_BYTES,
    ..RECORD_LIMITS
};

// Keep explicitly required optional fields in diagnostic JSON as well as Wire.
pub fn deserialize_present_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

pub fn deserialize_records<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    ic_wire::List::<T, { crate::constants::MAX_ALLOCATIONS }>::deserialize(deserializer)
        .map(ic_wire::List::into_vec)
}
