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
    SetU64,
    SetString,
    VectorU64,
    VectorBytes,
    VectorBytes32,
    VectorRecord,
    CellBytes,
    CellRecord,
    RegionBytes,
    I64I64,
    U128U128,
    Bytes64U128,
    U64Fixed31,
    U64Fixed32,
    U64Fixed33,
    U64Fixed256,
    SetI64,
    SetBytes64,
    VectorI64,
    VectorU128,
    VectorF64,
    CellU128,
    CellFixed32,
}

pub const CASES: [Case; 29] = [
    Case::U64Bytes,
    Case::U64U64,
    Case::U64U128,
    Case::Bytes32Bytes32,
    Case::StringString,
    Case::U64Record,
    Case::SetU64,
    Case::SetString,
    Case::VectorU64,
    Case::VectorBytes,
    Case::VectorBytes32,
    Case::VectorRecord,
    Case::CellBytes,
    Case::CellRecord,
    Case::RegionBytes,
    Case::I64I64,
    Case::U128U128,
    Case::Bytes64U128,
    Case::U64Fixed31,
    Case::U64Fixed32,
    Case::U64Fixed33,
    Case::U64Fixed256,
    Case::SetI64,
    Case::SetBytes64,
    Case::VectorI64,
    Case::VectorU128,
    Case::VectorF64,
    Case::CellU128,
    Case::CellFixed32,
];

impl Case {
    pub fn structure(self) -> &'static str {
        match self {
            Self::SetU64 | Self::SetString | Self::SetI64 | Self::SetBytes64 => "Set",
            Self::VectorU64
            | Self::VectorBytes
            | Self::VectorBytes32
            | Self::VectorRecord
            | Self::VectorI64
            | Self::VectorU128
            | Self::VectorF64 => "Vector",
            Self::CellBytes | Self::CellRecord | Self::CellU128 | Self::CellFixed32 => "Cell",
            Self::RegionBytes => "Region",
            _ => "Map",
        }
    }
    pub fn rows(self, rows: u32) -> u32 {
        if self.structure() == "Cell" { 1 } else { rows }
    }
    pub fn operations(self) -> Vec<super::Operation> {
        use super::Operation::*;
        match self.structure() {
            "Cell" => vec![Control, Get, Replace, Grow, Shrink],
            "Vector" => vec![Control, Get, MissAbove, Set, Scan, ReverseScan, Push, Pop],
            "Set" => vec![
                Control,
                Get,
                MissAbove,
                Scan,
                Insert,
                Remove,
                RemoveMiss,
                Reinsert,
                RemoveInserted,
            ],
            "Region" => vec![
                Control,
                Get,
                MissAbove,
                Replace,
                KeysScan,
                Insert,
                Grow,
                Shrink,
                Remove,
                RemoveMiss,
                Reinsert,
                RemoveInserted,
            ],
            _ => super::OPERATIONS.to_vec(),
        }
        .into_iter()
        .filter(|op| {
            !matches!(op, Grow | Shrink)
                || matches!(
                    self,
                    Self::U64Bytes | Self::StringString | Self::CellBytes | Self::RegionBytes
                )
        })
        .collect()
    }
    pub fn key_type(self) -> &'static str {
        match self {
            Self::Bytes32Bytes32 => "[u8;32]",
            Self::Bytes64U128 | Self::SetBytes64 => "[u8;64]",
            Self::I64I64 | Self::SetI64 => "Signed64(i64)",
            Self::U128U128 => "u128",
            Self::StringString | Self::SetString => "String",
            _ => "u64",
        }
    }
    pub fn value_type(self) -> &'static str {
        match self {
            Self::U64Bytes | Self::CellBytes | Self::RegionBytes => "Vec<u8>",
            Self::VectorBytes => "ChunkBytes(max=1MiB)",
            Self::U64U64 | Self::VectorU64 => "u64",
            Self::U64U128
            | Self::U128U128
            | Self::Bytes64U128
            | Self::VectorU128
            | Self::CellU128 => "u128",
            Self::I64I64 | Self::VectorI64 => "Signed64(i64)",
            Self::VectorF64 => "f64",
            Self::U64Fixed31 => "[u8;31]",
            Self::U64Fixed33 => "[u8;33]",
            Self::U64Fixed256 => "[u8;256]",
            Self::Bytes32Bytes32 => "[u8;32]",
            Self::StringString => "String",
            Self::U64Record | Self::VectorRecord | Self::CellRecord => "WireRecord",
            Self::SetU64 | Self::SetString | Self::SetI64 | Self::SetBytes64 => "()",
            Self::VectorBytes32 | Self::U64Fixed32 | Self::CellFixed32 => "[u8;32]",
        }
    }
    /// Larger byte records are opt-in through the consumer workload profiles.
    pub fn accepts_payload(self, payload: u32) -> bool {
        self.payloads().contains(&payload)
            || matches!(
                self,
                Self::U64Bytes | Self::CellBytes | Self::RegionBytes | Self::VectorBytes
            ) && [
                44, 45, 80, 108, 109, 132, 235, 236, 237, 256, 4075, 4076, 4077, 65_515, 65_516,
                65_517, 65_536, 1_048_576,
            ]
            .contains(&payload)
    }
    /// Logical value sizes; WireRecord has 60 field bytes, with variable encoding.
    pub fn payloads(self) -> &'static [u32] {
        match self {
            Self::U64Bytes | Self::CellBytes | Self::RegionBytes | Self::VectorBytes => {
                &[0, 31, 32, 33, 1024, 4096]
            }
            Self::StringString => &[0, 32, 1024, 4096],
            Self::I64I64 | Self::VectorI64 | Self::VectorF64 => &[8],
            Self::U64Fixed31 => &[31],
            Self::U64Fixed33 => &[33],
            Self::U64Fixed256 => &[256],
            Self::U64U64 | Self::VectorU64 => &[8],
            Self::U64U128
            | Self::U128U128
            | Self::Bytes64U128
            | Self::VectorU128
            | Self::CellU128 => &[16],
            Self::Bytes32Bytes32 | Self::VectorBytes32 | Self::U64Fixed32 | Self::CellFixed32 => {
                &[32]
            }
            Self::U64Record | Self::VectorRecord | Self::CellRecord => &[60],
            Self::SetU64 | Self::SetString | Self::SetI64 | Self::SetBytes64 => &[0],
        }
    }
    pub fn expected_value(self, key: u64, payload: u32, replacement: bool) -> Vec<u8> {
        match self {
            Self::U64Bytes | Self::CellBytes | Self::RegionBytes | Self::VectorBytes => {
                Vec::<u8>::make(key, payload, replacement).canonical()
            }
            Self::U64U64 | Self::VectorU64 => u64::make(key, payload, replacement).canonical(),
            Self::U64U128
            | Self::U128U128
            | Self::Bytes64U128
            | Self::VectorU128
            | Self::CellU128 => u128::make(key, payload, replacement).canonical(),
            Self::I64I64 | Self::VectorI64 => i64::make(key, payload, replacement).canonical(),
            Self::VectorF64 => f64::make(key, payload, replacement).canonical(),
            Self::U64Fixed31 => <[u8; 31]>::make(key, payload, replacement).canonical(),
            Self::U64Fixed33 => <[u8; 33]>::make(key, payload, replacement).canonical(),
            Self::U64Fixed256 => <[u8; 256]>::make(key, payload, replacement).canonical(),
            Self::Bytes32Bytes32 | Self::VectorBytes32 | Self::U64Fixed32 | Self::CellFixed32 => {
                <[u8; 32]>::make(key, payload, replacement).canonical()
            }
            Self::StringString => String::make(key, payload, replacement).canonical(),
            Self::U64Record | Self::VectorRecord | Self::CellRecord => {
                WireRecord::make(key, payload, replacement).canonical()
            }
            Self::SetU64 | Self::SetString | Self::SetI64 | Self::SetBytes64 => vec![],
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
///
/// Signed64
///
/// Signed integer with the same fixed-width codec on both backends; upstream
/// stable-structures does not implement Storable for primitive i64.
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Signed64(i64);
impl BenchKey for Signed64 {
    fn from_id(id: u64) -> Self {
        Self(i64::from_id(id))
    }
    fn id(&self) -> u64 {
        self.0.id()
    }
}
impl BenchValue for Signed64 {
    fn make(key: u64, payload: u32, replacement: bool) -> Self {
        Self(i64::make(key, payload, replacement))
    }
    fn canonical(self) -> Vec<u8> {
        self.0.canonical()
    }
}
impl BenchKey for i64 {
    fn from_id(id: u64) -> Self {
        i64::try_from(id).expect("bounded benchmark key") - 32
    }
    fn id(&self) -> u64 {
        u64::try_from(*self + 32).unwrap()
    }
}
impl BenchKey for u128 {
    fn from_id(id: u64) -> Self {
        (u128::from(id) << 64) | 0xa5a5_0123_4567_89ab
    }
    fn id(&self) -> u64 {
        let id = (self >> 64) as u64;
        assert_eq!(*self, Self::from_id(id));
        id
    }
}
impl BenchKey for [u8; 64] {
    fn from_id(id: u64) -> Self {
        let mut key = [0x5a; 64];
        key[56..].copy_from_slice(&id.to_be_bytes());
        key
    }
    fn id(&self) -> u64 {
        let id = u64::from_be_bytes(self[56..].try_into().unwrap());
        assert_eq!(*self, Self::from_id(id));
        id
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
pub trait BenchValue: Clone {
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
impl BenchValue for i64 {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        i64::from_le_bytes(u64::make(key, 8, replacement).to_le_bytes())
    }
    fn canonical(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
impl BenchValue for f64 {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        // Include signed zero, subnormals, infinities and a NaN payload.
        let bits = [
            0,
            1 << 63,
            1,
            (1 << 63) | 1,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_0042,
            0x3ff8_0000_0000_0000,
        ][key as usize % 8];
        Self::from_bits(bits ^ if replacement { 1 << 63 } else { 0 })
    }
    fn canonical(self) -> Vec<u8> {
        self.to_bits().to_le_bytes().to_vec()
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
impl<const N: usize> BenchValue for [u8; N] {
    fn make(key: u64, _: u32, replacement: bool) -> Self {
        super::value(key, u32::try_from(N).unwrap(), replacement)
            .try_into()
            .unwrap()
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
/// ChunkBytes
///
/// Bounded raw chunk bytes, matching Canic's 1 MiB variable-length vector contract.
///
#[derive(Clone, Debug)]
pub struct ChunkBytes(Vec<u8>);
impl BenchValue for ChunkBytes {
    fn make(key: u64, payload: u32, replacement: bool) -> Self {
        assert!(payload <= 1_048_576);
        Self(super::value(key, payload, replacement))
    }
    fn canonical(self) -> Vec<u8> {
        self.0
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
    use super::{ChunkBytes, Signed64, WireRecord};
    #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
    use crate::upstream::{Storable, storable::Bound};
    #[cfg(feature = "owned")]
    use ic_storage::adapter::{Bound, CodecError, Storable};
    use std::borrow::Cow;
    impl Storable for ChunkBytes {
        const BOUND: Bound = Bound::Bounded {
            max_size: 1_048_576,
            is_fixed_size: false,
        };
        #[cfg(feature = "owned")]
        fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
            Self::check_bounds(&self.0)?;
            Ok(Cow::Borrowed(&self.0))
        }
        #[cfg(feature = "owned")]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
            Self::check_bounds(&bytes)?;
            Ok(Self(bytes.into_owned()))
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn to_bytes(&self) -> Cow<'_, [u8]> {
            Cow::Borrowed(&self.0)
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn into_bytes(self) -> Vec<u8> {
            self.0
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
            Self(bytes.into_owned())
        }
    }
    impl Storable for Signed64 {
        const BOUND: Bound = Bound::Bounded {
            max_size: 8,
            is_fixed_size: true,
        };
        #[cfg(feature = "owned")]
        fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
            Ok(Cow::Owned(self.0.to_le_bytes().to_vec()))
        }
        #[cfg(feature = "owned")]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
            Ok(Self(i64::from_le_bytes(
                bytes
                    .as_ref()
                    .try_into()
                    .map_err(|_| CodecError::Invalid("benchmark signed width"))?,
            )))
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn to_bytes(&self) -> Cow<'_, [u8]> {
            Cow::Owned(self.0.to_le_bytes().to_vec())
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn into_bytes(self) -> Vec<u8> {
            self.0.to_le_bytes().to_vec()
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
            Self(i64::from_le_bytes(bytes.as_ref().try_into().unwrap()))
        }
    }
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
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn to_bytes(&self) -> Cow<'_, [u8]> {
            Cow::Owned(self.encode().unwrap())
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn into_bytes(self) -> Vec<u8> {
            self.encode().unwrap()
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
            Self::decode(&bytes).unwrap()
        }
    }
}

#[cfg(test)]
mod tests;
