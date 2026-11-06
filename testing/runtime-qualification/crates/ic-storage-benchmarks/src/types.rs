//! Concrete map types and canonical results used only by the benchmark.
use serde::{Deserialize, Serialize};
mod breadth;
pub use breadth::{BoundedBytes, OptionalU64, TupleKey};

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
    VectorSession,
    U32U32,
    U64Unit,
    U64BoundedBytes,
    BoundedBytesU64,
    BoundedBytesBoundedBytes,
    BoundedBytesBytes,
    BytesU64,
    BytesBoundedBytes,
    BytesBytes,
    TupleOption,
    SetBytes,
    SetBoundedBytes,
    SetTuple,
    VectorUnit,
    VectorU8,
    VectorU16,
    VectorU32,
    VectorBool,
    VectorF32,
    VectorOption,
    VectorBounded255,
    VectorBounded256,
    VectorBounded65535,
    VectorBounded65536,
    CellString,
    CellUnit,
    CellBool,
    CellOption,
    Typed {
        collection: crate::Collection,
        key: crate::DataType,
        value: crate::DataType,
    },
}

pub const CASES: [Case; 58] = [
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
    Case::VectorSession,
    Case::U32U32,
    Case::U64Unit,
    Case::U64BoundedBytes,
    Case::BoundedBytesU64,
    Case::BoundedBytesBoundedBytes,
    Case::BoundedBytesBytes,
    Case::BytesU64,
    Case::BytesBoundedBytes,
    Case::BytesBytes,
    Case::TupleOption,
    Case::SetBytes,
    Case::SetBoundedBytes,
    Case::SetTuple,
    Case::VectorUnit,
    Case::VectorU8,
    Case::VectorU16,
    Case::VectorU32,
    Case::VectorBool,
    Case::VectorF32,
    Case::VectorOption,
    Case::VectorBounded255,
    Case::VectorBounded256,
    Case::VectorBounded65535,
    Case::VectorBounded65536,
    Case::CellString,
    Case::CellUnit,
    Case::CellBool,
    Case::CellOption,
];

/// Curated ordinary trace; the `types` preset selects the complete registry.
/// Keep its established runtime while growing opt-in type qualification.
pub const DEFAULT_CASES: &[Case] = &[
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
    Case::VectorSession,
];

impl Case {
    pub fn structure(self) -> &'static str {
        if let Self::Typed { collection, .. } = self {
            return collection.name();
        }

        match self {
            Self::SetU64
            | Self::SetString
            | Self::SetI64
            | Self::SetBytes64
            | Self::SetBytes
            | Self::SetBoundedBytes
            | Self::SetTuple => "Set",
            Self::VectorUnit
            | Self::VectorU8
            | Self::VectorU16
            | Self::VectorU32
            | Self::VectorBool
            | Self::VectorF32
            | Self::VectorOption
            | Self::VectorBounded255
            | Self::VectorBounded256
            | Self::VectorBounded65535
            | Self::VectorBounded65536
            | Self::VectorSession
            | Self::VectorU64
            | Self::VectorBytes
            | Self::VectorBytes32
            | Self::VectorRecord
            | Self::VectorI64
            | Self::VectorU128
            | Self::VectorF64 => "Vector",
            Self::CellBytes
            | Self::CellRecord
            | Self::CellU128
            | Self::CellFixed32
            | Self::CellString
            | Self::CellUnit
            | Self::CellBool
            | Self::CellOption => "Cell",
            Self::RegionBytes => "Region",
            _ => "Map",
        }
    }
    pub fn rows(self, rows: u32) -> u32 {
        if let Self::Typed {
            collection, key, ..
        } = self
        {
            return if collection == crate::Collection::Cell {
                1
            } else {
                rows.min(key.population_limit())
            };
        }

        if self.structure() == "Cell" { 1 } else { rows }
    }
    pub fn operations(self) -> Vec<super::Operation> {
        if matches!(self, Self::Typed { .. }) {
            return crate::catalogue::operations(self);
        }

        use super::Operation::*;
        if self == Self::VectorSession {
            return vec![Control, Get, WarmGet, Decode, MissAbove];
        }
        match self.structure() {
            "Cell" => vec![Control, Get, Replace, Grow, Shrink],
            "Vector" => vec![
                Control,
                Get,
                MissAbove,
                Set,
                Scan,
                ReverseScan,
                Nth,
                NthBack,
                Last,
                AlternatingScan,
                ScanSet,
                Push,
                Pop,
            ],
            "Set" => vec![
                Control,
                Get,
                MissAbove,
                Scan,
                Nth,
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
                    Self::U64Bytes
                        | Self::StringString
                        | Self::CellBytes
                        | Self::RegionBytes
                        | Self::BoundedBytesBytes
                        | Self::BytesBytes
                        | Self::CellString
                )
        })
        .collect()
    }
    pub fn key_type(self) -> &'static str {
        if let Self::Typed { key, .. } = self {
            return key.name();
        }

        match self {
            Self::U32U32 => "u32",
            Self::BoundedBytesU64
            | Self::BoundedBytesBoundedBytes
            | Self::BoundedBytesBytes
            | Self::SetBoundedBytes => "BoundedBytes<255>",
            Self::BytesU64 | Self::BytesBoundedBytes | Self::BytesBytes | Self::SetBytes => {
                "Vec<u8>"
            }
            Self::TupleOption | Self::SetTuple => "TupleKey(u64,[u8;20])",
            Self::Bytes32Bytes32 => "[u8;32]",
            Self::Bytes64U128 | Self::SetBytes64 => "[u8;64]",
            Self::I64I64 | Self::SetI64 => "Signed64(i64)",
            Self::U128U128 => "u128",
            Self::StringString | Self::SetString => "String",
            _ => "u64",
        }
    }
    pub fn value_type(self) -> &'static str {
        if let Self::Typed { value, .. } = self {
            return value.name();
        }

        match self {
            Self::U32U32 | Self::VectorU32 => "u32",
            Self::VectorU8 => "u8",
            Self::VectorU16 => "u16",
            Self::VectorBool | Self::CellBool => "bool",
            Self::VectorF32 => "f32",
            Self::U64Unit
            | Self::VectorUnit
            | Self::CellUnit
            | Self::SetBytes
            | Self::SetBoundedBytes
            | Self::SetTuple => "()",
            Self::BoundedBytesU64 | Self::BytesU64 => "u64",
            Self::TupleOption | Self::VectorOption | Self::CellOption => {
                "OptionalU64(variable bound=9)"
            }
            Self::U64BoundedBytes
            | Self::BoundedBytesBoundedBytes
            | Self::BytesBoundedBytes
            | Self::VectorBounded255 => "BoundedBytes<255>",
            Self::VectorBounded256 => "BoundedBytes<256>",
            Self::VectorBounded65535 => "BoundedBytes<65535>",
            Self::VectorBounded65536 => "BoundedBytes<65536>",
            Self::CellString => "String",
            Self::BoundedBytesBytes | Self::BytesBytes => "Vec<u8>",
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
            Self::VectorSession => "Wire byte-envelope (bound=4096)",
            Self::U64Fixed31 => "[u8;31]",
            Self::U64Fixed33 => "[u8;33]",
            Self::U64Fixed256 => "[u8;256]",
            Self::Bytes32Bytes32 => "[u8;32]",
            Self::StringString => "String",
            Self::U64Record | Self::VectorRecord | Self::CellRecord => "WireRecord",
            Self::SetU64 | Self::SetString | Self::SetI64 | Self::SetBytes64 => "()",
            Self::VectorBytes32 | Self::U64Fixed32 | Self::CellFixed32 => "[u8;32]",
            Self::Typed { .. } => unreachable!(),
        }
    }
    /// Stored-key geometry; Vector/Cell use logical indices, not stored keys.
    pub fn key_bound(self) -> &'static str {
        if let Self::Typed {
            collection, key, ..
        } = self
        {
            return if matches!(
                collection,
                crate::Collection::Cell | crate::Collection::Vector
            ) {
                "index"
            } else {
                key.bound()
            };
        }

        match self {
            Self::BoundedBytesU64
            | Self::BoundedBytesBoundedBytes
            | Self::BoundedBytesBytes
            | Self::SetBoundedBytes => "bounded-variable",
            Self::BytesU64
            | Self::BytesBoundedBytes
            | Self::BytesBytes
            | Self::SetBytes
            | Self::StringString
            | Self::SetString => "unbounded",
            _ if matches!(self.structure(), "Vector" | "Cell") => "index",
            _ => "fixed",
        }
    }
    /// Declared value geometry, independently of the generated payload length.
    pub fn value_bound(self) -> &'static str {
        if let Self::Typed { value, .. } = self {
            return value.bound();
        }

        match self {
            Self::U64Bytes
            | Self::CellBytes
            | Self::RegionBytes
            | Self::StringString
            | Self::BoundedBytesBytes
            | Self::BytesBytes
            | Self::CellString => "unbounded",
            Self::U64BoundedBytes
            | Self::BoundedBytesBoundedBytes
            | Self::BytesBoundedBytes
            | Self::VectorBytes
            | Self::VectorRecord
            | Self::U64Record
            | Self::CellRecord
            | Self::VectorSession
            | Self::TupleOption
            | Self::VectorOption
            | Self::CellOption
            | Self::VectorBounded255
            | Self::VectorBounded256
            | Self::VectorBounded65535
            | Self::VectorBounded65536 => "bounded-variable",
            _ => "fixed",
        }
    }
    /// Larger byte records are opt-in through the consumer workload profiles.
    pub fn accepts_payload(self, payload: u32) -> bool {
        if let Self::Typed { value, .. } = self {
            return value.payloads().contains(&payload);
        }

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
        if let Self::Typed { value, .. } = self {
            return value.payloads();
        }

        match self {
            Self::VectorUnit
            | Self::CellUnit
            | Self::U64Unit
            | Self::SetBytes
            | Self::SetBoundedBytes
            | Self::SetTuple => &[0],
            Self::VectorU8 | Self::VectorBool | Self::CellBool => &[1],
            Self::VectorU16 => &[2],
            Self::U32U32 | Self::VectorU32 | Self::VectorF32 => &[4],
            Self::BoundedBytesU64 | Self::BytesU64 => &[8],
            Self::TupleOption | Self::VectorOption | Self::CellOption => &[9],
            Self::U64BoundedBytes
            | Self::BoundedBytesBoundedBytes
            | Self::BytesBoundedBytes
            | Self::VectorBounded255 => &[0, 8, 255],
            Self::VectorBounded256 => &[0, 8, 256],
            Self::VectorBounded65535 | Self::VectorBounded65536 => &[0, 8, 256],
            Self::BoundedBytesBytes | Self::BytesBytes => &[0, 32, 256],
            Self::CellString => &[0, 32, 1024],
            Self::VectorSession => &[256, 2048],
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
            Self::Typed { .. } => unreachable!(),
        }
    }
    pub fn expected_value(self, key: u64, payload: u32, replacement: bool) -> Vec<u8> {
        if let Self::Typed { value, .. } = self {
            return value.expected_value(key, payload, replacement);
        }

        match self {
            Self::VectorUnit
            | Self::CellUnit
            | Self::U64Unit
            | Self::SetBytes
            | Self::SetBoundedBytes
            | Self::SetTuple => vec![],
            Self::VectorU8 => u8::make(key, payload, replacement).canonical(),
            Self::VectorU16 => u16::make(key, payload, replacement).canonical(),
            Self::U32U32 | Self::VectorU32 => u32::make(key, payload, replacement).canonical(),
            Self::VectorBool | Self::CellBool => bool::make(key, payload, replacement).canonical(),
            Self::VectorF32 => f32::make(key, payload, replacement).canonical(),
            Self::BoundedBytesU64 | Self::BytesU64 => {
                u64::make(key, payload, replacement).canonical()
            }
            Self::TupleOption | Self::VectorOption | Self::CellOption => {
                OptionalU64::make(key, payload, replacement).canonical()
            }
            Self::U64BoundedBytes
            | Self::BoundedBytesBoundedBytes
            | Self::BytesBoundedBytes
            | Self::VectorBounded255
            | Self::VectorBounded256
            | Self::VectorBounded65535
            | Self::VectorBounded65536
            | Self::BoundedBytesBytes
            | Self::BytesBytes => crate::value(key, payload, replacement),
            Self::CellString => String::make(key, payload, replacement).canonical(),
            Self::VectorSession => SessionEnvelope::make(key, payload, replacement).canonical(),
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
            Self::Typed { .. } => unreachable!(),
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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
    use super::{ChunkBytes, SessionEnvelope, Signed64, WireRecord, measured_session_decode};
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
    impl Storable for SessionEnvelope {
        const BOUND: Bound = Bound::Bounded {
            max_size: 4096,
            is_fixed_size: false,
        };
        #[cfg(feature = "owned")]
        fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
            self.encode()
                .map(Cow::Owned)
                .map_err(|_| CodecError::Invalid("session envelope"))
        }
        #[cfg(feature = "owned")]
        fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
            measured_session_decode(&bytes).map_err(|_| CodecError::Invalid("session envelope"))
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
            measured_session_decode(&bytes).unwrap()
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

///
/// SessionEnvelope
///
/// Canic-shaped outer storage record; inner Auth bytes remain opaque synthetic data.
///

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SessionEnvelope {
    #[serde(with = "serde_bytes")]
    session: Vec<u8>,
}
impl SessionEnvelope {
    pub fn encode(&self) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode(&(*b"CSW\x01", self), SESSION_LIMITS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ic_wire::Error> {
        if !bytes.starts_with(b"CSW\x01") {
            return Err(ic_wire::Error::Identity);
        }
        ic_wire::native::decode::<([u8; 4], Self)>(bytes, SESSION_LIMITS).map(|(_, value)| value)
    }
}
const SESSION_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 64 * 1024 * 1024,
    allocation: 1024 * 1024 * 1024,
    nodes: 64 * 1024 * 1024,
    depth: 128,
};
impl BenchValue for SessionEnvelope {
    fn make(key: u64, payload: u32, replacement: bool) -> Self {
        let mut session = crate::value(key, payload, replacement);
        session[..8].copy_from_slice(&key.to_le_bytes());
        Self { session }
    }
    fn canonical(self) -> Vec<u8> {
        self.session
    }
}
#[cfg(target_arch = "wasm32")]
thread_local! { static SESSION_DECODE: std::cell::Cell<crate::DecodeMetrics> = const { std::cell::Cell::new(crate::DecodeMetrics { calls:0,bytes:0,instructions:0 }) }; }
#[cfg(target_arch = "wasm32")]
pub fn reset_decode_metrics() {
    SESSION_DECODE.with(|m| m.set(crate::DecodeMetrics::default()));
}
#[cfg(target_arch = "wasm32")]
pub fn decode_metrics() -> crate::DecodeMetrics {
    SESSION_DECODE.with(std::cell::Cell::get)
}
#[cfg(target_arch = "wasm32")]
fn measured_session_decode(bytes: &[u8]) -> Result<SessionEnvelope, ic_wire::Error> {
    let start = ic_platform_runtime::ic0::performance_counter(0);
    let decoded = SessionEnvelope::decode(bytes);
    let cost = ic_platform_runtime::ic0::performance_counter(0) - start;
    SESSION_DECODE.with(|m| {
        let mut metrics = m.get();
        metrics.calls += 1;
        metrics.bytes += bytes.len() as u64;
        metrics.instructions += cost;
        m.set(metrics);
    });
    decoded
}

impl BenchKey for WireRecord {
    fn from_id(id: u64) -> Self {
        Self::make(id, 60, false)
    }
    fn id(&self) -> u64 {
        assert_eq!(*self, Self::from_id(self.account));
        self.account
    }
}
impl BenchKey for SessionEnvelope {
    fn from_id(id: u64) -> Self {
        let mut session = id.to_be_bytes().to_vec();
        session.resize(32, 0x5a);
        Self { session }
    }
    fn id(&self) -> u64 {
        let id = u64::from_be_bytes(self.session[..8].try_into().unwrap());
        assert_eq!(*self, Self::from_id(id));
        id
    }
}
impl BenchKey for ChunkBytes {
    fn from_id(id: u64) -> Self {
        Self(Vec::<u8>::from_id(id))
    }
    fn id(&self) -> u64 {
        self.0.id()
    }
}
