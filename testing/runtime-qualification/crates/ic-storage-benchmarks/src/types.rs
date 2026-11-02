//! Concrete map types and canonical results used only by the benchmark.
use serde::{Deserialize, Serialize};

///
/// Case
///
/// Concrete key/value pair selected once when a fresh canister is installed.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Case {
    U64Bytes,
    U64U64,
    U64U128,
    Bytes32Bytes32,
    StringString,
    U64Record,
}

pub const CASES: [Case; 6] = [
    Case::U64Bytes,
    Case::U64U64,
    Case::U64U128,
    Case::Bytes32Bytes32,
    Case::StringString,
    Case::U64Record,
];

impl Case {
    pub fn key_type(self) -> &'static str {
        match self {
            Self::Bytes32Bytes32 => "[u8;32]",
            Self::StringString => "String",
            _ => "u64",
        }
    }
    pub fn value_type(self) -> &'static str {
        match self {
            Self::U64Bytes => "Vec<u8>",
            Self::U64U64 => "u64",
            Self::U64U128 => "u128",
            Self::Bytes32Bytes32 => "[u8;32]",
            Self::StringString => "String",
            Self::U64Record => "WireRecord",
        }
    }
    /// Logical value sizes; WireRecord has 60 field bytes, with variable encoding.
    pub fn payloads(self) -> &'static [u32] {
        match self {
            Self::U64Bytes | Self::StringString => &[32, 1024],
            Self::U64U64 => &[8],
            Self::U64U128 => &[16],
            Self::Bytes32Bytes32 => &[32],
            Self::U64Record => &[60],
        }
    }
    pub fn expected_value(self, key: u64, payload: u32, replacement: bool) -> Vec<u8> {
        assert!(self.payloads().contains(&payload));
        match self {
            Self::U64Bytes => Vec::<u8>::make(key, payload, replacement).canonical(),
            Self::U64U64 => u64::make(key, payload, replacement).canonical(),
            Self::U64U128 => u128::make(key, payload, replacement).canonical(),
            Self::Bytes32Bytes32 => <[u8; 32]>::make(key, payload, replacement).canonical(),
            Self::StringString => String::make(key, payload, replacement).canonical(),
            Self::U64Record => WireRecord::make(key, payload, replacement).canonical(),
        }
    }
}

///
/// BenchKey
///
/// Order-preserving mapping from logical trace IDs to concrete map keys.
///
pub trait BenchKey: Ord + Clone {
    fn from_id(id: u64) -> Self;
    fn id(&self) -> u64;
}
impl BenchKey for u64 {
    fn from_id(id: u64) -> Self {
        id
    }
    fn id(&self) -> u64 {
        *self
    }
}
impl BenchKey for [u8; 32] {
    fn from_id(id: u64) -> Self {
        let mut key = [0x5a; 32];
        key[..8].copy_from_slice(&id.to_be_bytes());
        key
    }
    fn id(&self) -> u64 {
        let id = u64::from_be_bytes(self[..8].try_into().unwrap());
        assert_eq!(*self, Self::from_id(id));
        id
    }
}
impl BenchKey for String {
    fn from_id(id: u64) -> Self {
        format!("account/{id:020}")
    }
    fn id(&self) -> u64 {
        let id = self.strip_prefix("account/").unwrap().parse().unwrap();
        assert_eq!(*self, Self::from_id(id));
        id
    }
}

///
/// BenchValue
///
/// Generate concrete values before timing and normalize results after timing.
///
pub trait BenchValue {
    fn make(key: u64, payload: u32, replacement: bool) -> Self;
    fn canonical(self) -> Vec<u8>;
}
impl BenchValue for Vec<u8> {
    fn make(key: u64, payload: u32, replacement: bool) -> Self {
        super::value(key, payload, replacement)
    }
    fn canonical(self) -> Vec<u8> {
        self
    }
}
impl BenchValue for u64 {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        key ^ 0xa5a5_0000_0000_0000 ^ if replacement { 1 << 63 } else { 0 }
    }
    fn canonical(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
impl BenchValue for u128 {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        (u128::from(key) << 64) | u128::from(u64::make(key, 8, replacement))
    }
    fn canonical(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
impl BenchValue for [u8; 32] {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        super::value(key, 32, replacement).try_into().unwrap()
    }
    fn canonical(self) -> Vec<u8> {
        self.to_vec()
    }
}
impl BenchValue for String {
    fn make(key: u64, payload: u32, replacement: bool) -> Self {
        // Include non-ASCII UTF-8 while preserving the declared byte size.
        let letter =
            char::from_u32(0xe0 + (key % 16) as u32 + u32::from(replacement) * 16).unwrap();
        letter.to_string().repeat(payload as usize / 2)
    }
    fn canonical(self) -> Vec<u8> {
        self.into_bytes()
    }
}

///
/// WireRecord
///
/// Application-shaped record with the same bounded IC Wire codec on both stacks.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WireRecord {
    account: u64,
    balance: u128,
    generation: u32,
    tag: [u8; 32],
}
impl WireRecord {
    pub fn encode(&self) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode(self, RECORD_LIMITS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ic_wire::Error> {
        ic_wire::native::decode(bytes, RECORD_LIMITS)
    }
}
const RECORD_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 256,
    allocation: 16 * 1024,
    nodes: 256,
    depth: 8,
};
impl BenchValue for WireRecord {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        Self {
            account: key,
            balance: u128::make(key, 16, replacement),
            generation: u32::from(replacement),
            tag: <[u8; 32]>::make(key, 32, replacement),
        }
    }
    fn canonical(self) -> Vec<u8> {
        // Independent of the storage codec: validate every decoded field.
        let mut bytes = self.account.to_le_bytes().to_vec();
        bytes.extend(self.balance.to_le_bytes());
        bytes.extend(self.generation.to_le_bytes());
        bytes.extend(self.tag);
        bytes
    }
}

#[cfg(target_arch = "wasm32")]
mod codec {
    use super::WireRecord;
    #[cfg(feature = "dfinity")]
    use ic_platform::stable_structures::{Storable, storable::Bound};
    #[cfg(feature = "owned")]
    use ic_storage::adapter::{Bound, CodecError, Storable};
    use std::borrow::Cow;
    impl Storable for WireRecord {
        const BOUND: Bound = Bound::Bounded {
            max_size: 256,
            is_fixed_size: false,
        };
        #[cfg(feature = "owned")]
        fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
            self.encode()
                .map(Cow::Owned)
                .map_err(|_| CodecError::Invalid("benchmark Wire record"))
        }
        #[cfg(feature = "owned")]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
            Self::decode(&bytes).map_err(|_| CodecError::Invalid("benchmark Wire record"))
        }
        #[cfg(feature = "dfinity")]
        fn to_bytes(&self) -> Cow<'_, [u8]> {
            Cow::Owned(self.encode().unwrap())
        }
        #[cfg(feature = "dfinity")]
        fn into_bytes(self) -> Vec<u8> {
            self.encode().unwrap()
        }
        #[cfg(feature = "dfinity")]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
            Self::decode(&bytes).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check_keys<K: BenchKey + std::fmt::Debug>() {
        let ids = [0, 1, 9, 10, 255, 256, 16_415, u64::MAX];
        let keys: Vec<_> = ids.into_iter().map(K::from_id).collect();
        assert!(keys.windows(2).all(|p| p[0] < p[1]));
        assert_eq!(keys.iter().map(K::id).collect::<Vec<_>>(), ids);
    }
    #[test]
    fn concrete_keys_preserve_order_and_identity() {
        check_keys::<u64>();
        check_keys::<[u8; 32]>();
        check_keys::<String>();
    }
    #[test]
    fn values_cover_declared_sizes_and_record_fields() {
        for case in CASES {
            for &payload in case.payloads() {
                for key in [0, 255, 16_415, u64::MAX] {
                    let before = case.expected_value(key, payload, false);
                    let after = case.expected_value(key, payload, true);
                    assert_eq!(before.len(), payload as usize);
                    assert_eq!(after.len(), payload as usize);
                    assert_ne!(before, after);
                    if case == Case::StringString {
                        assert!(
                            std::str::from_utf8(&before)
                                .unwrap()
                                .contains(|c: char| !c.is_ascii())
                        );
                    }
                }
            }
        }
        for replacement in [false, true] {
            let record = WireRecord::make(u64::MAX, 60, replacement);
            let encoded = record.encode().unwrap();
            assert!(encoded.len() <= 256);
            assert_eq!(WireRecord::decode(&encoded).unwrap(), record);
            assert_eq!(&record.clone().canonical()[..8], &u64::MAX.to_le_bytes());
            assert_eq!(
                &record.canonical()[24..28],
                &u32::from(replacement).to_le_bytes()
            );
        }
    }
}
