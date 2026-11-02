//! Storage-owned record codec contract. Application records normally use IC Wire.
use std::borrow::Cow;

///
/// Bound
///
/// Maximum encoded record size, with an optional exact-width requirement.
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bound {
    /// No type-level bound; collection resource limits still apply.
    Unbounded,
    /// A finite encoded-size limit.
    Bounded {
        /// Maximum encoded byte count.
        max_size: u32,
        /// Whether every encoding must have exactly the maximum length.
        is_fixed_size: bool,
    },
}

///
/// CodecError
///
/// Invalid application encoding or a violation of its declared byte bound.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CodecError {
    /// The payload does not satisfy the codec.
    #[error("invalid encoding: {0}")]
    Invalid(&'static str),
    /// Encoded bytes violate the declared maximum or fixed width.
    #[error("record violates codec bound")]
    Bounds,
}

///
/// Storable
///
/// Encode and decode a collection record. The caller owns its schema; Storage
/// enforces its declared byte bound before writes and after reading bytes.
///
pub trait Storable: Sized {
    /// Record size contract.
    const BOUND: Bound;
    /// Borrow or allocate an encoding.
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError>;
    /// Consume a value into its encoding.
    fn into_bytes(self) -> Result<Vec<u8>, CodecError> {
        Ok(self.to_bytes()?.into_owned())
    }
    /// Decode application bytes, returning an error for invalid encodings.
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError>;
    /// Enforce the declared maximum and exact width where applicable.
    fn check_bounds(bytes: &[u8]) -> Result<(), CodecError> {
        if let Bound::Bounded {
            max_size,
            is_fixed_size,
        } = Self::BOUND
            && !(bytes.len() <= max_size as usize
                && (!is_fixed_size || bytes.len() == max_size as usize))
        {
            return Err(CodecError::Bounds);
        }
        Ok(())
    }
    /// Encode and enforce the declared bound.
    fn to_bytes_checked(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        let bytes = self.to_bytes()?;
        Self::check_bounds(&bytes)?;
        Ok(bytes)
    }
    /// Consume, encode and enforce the declared bound.
    fn into_bytes_checked(self) -> Result<Vec<u8>, CodecError> {
        let bytes = self.into_bytes()?;
        Self::check_bounds(&bytes)?;
        Ok(bytes)
    }
}

macro_rules! integer {
    ($($ty:ty),* $(,)?) => {$ (
        impl Storable for $ty {
            #[expect(clippy::cast_possible_truncation, reason = "primitive widths are at most 16 bytes")]
            const BOUND: Bound = Bound::Bounded { max_size: size_of::<Self>() as u32, is_fixed_size: true };
            fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> { Ok(Cow::Owned(self.to_le_bytes().to_vec())) }
            fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> { Ok(Self::from_le_bytes(bytes.as_ref().try_into().map_err(|_| CodecError::Invalid("integer width"))?)) }
        }
    )*};
}
integer!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
impl Storable for () {
    const BOUND: Bound = Bound::Bounded {
        max_size: 0,
        is_fixed_size: true,
    };
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Borrowed(&[]))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        Self::check_bounds(&bytes)
    }
}
impl Storable for bool {
    const BOUND: Bound = Bound::Bounded {
        max_size: 1,
        is_fixed_size: true,
    };
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Owned(vec![u8::from(*self)]))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        match bytes.as_ref() {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err(CodecError::Invalid("boolean encoding")),
        }
    }
}
impl Storable for Vec<u8> {
    const BOUND: Bound = Bound::Unbounded;
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Borrowed(self))
    }
    fn into_bytes(self) -> Result<Vec<u8>, CodecError> {
        Ok(self)
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        Ok(bytes.into_owned())
    }
}
impl Storable for String {
    const BOUND: Bound = Bound::Unbounded;
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Borrowed(self.as_bytes()))
    }
    fn into_bytes(self) -> Result<Vec<u8>, CodecError> {
        Ok(self.into_bytes())
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        Self::from_utf8(bytes.into_owned()).map_err(|_| CodecError::Invalid("UTF-8 record"))
    }
}
impl<const N: usize> Storable for [u8; N] {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the const assertion admits only u32 widths"
    )]
    const BOUND: Bound = Bound::Bounded {
        max_size: {
            assert!(N <= u32::MAX as usize);
            N as u32
        },
        is_fixed_size: true,
    };
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Borrowed(self))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        bytes
            .as_ref()
            .try_into()
            .map_err(|_| CodecError::Invalid("byte array width"))
    }
}
impl<T: Storable> Storable for Option<T> {
    const BOUND: Bound = match T::BOUND {
        Bound::Unbounded => Bound::Unbounded,
        Bound::Bounded { max_size, .. } => Bound::Bounded {
            max_size: max_size.checked_add(1).expect("optional bound overflow"),
            is_fixed_size: false,
        },
    };
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        let mut bytes = vec![u8::from(self.is_some())];
        if let Some(value) = self {
            bytes.extend_from_slice(&value.to_bytes_checked()?);
        }
        Ok(Cow::Owned(bytes))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        match bytes.as_ref().split_first() {
            Some((0, [])) => Ok(None),
            Some((1, payload)) => {
                T::check_bounds(payload)?;
                T::from_bytes(Cow::Borrowed(payload)).map(Some)
            }
            _ => Err(CodecError::Invalid("optional record tag")),
        }
    }
}
