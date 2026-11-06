// Adapted from ic-stable-structures 0.7.2, DFINITY Foundation.
// SPDX-License-Identifier: Apache-2.0
// See the package LICENSE-APACHE and docs/storage/icss-source.json.
use std::borrow::Cow;
pub trait Storable {
    fn to_bytes(&self) -> Cow<'_, [u8]>;
    fn into_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: Cow<[u8]>) -> Self;
    const BOUND: Bound;
    fn to_bytes_checked(&self) -> Cow<'_, [u8]> {
        let bytes = self.to_bytes();
        Self::check_bounds(&bytes);
        bytes
    }
    fn into_bytes_checked(self) -> Vec<u8>
    where
        Self: Sized,
    {
        let bytes = self.into_bytes();
        Self::check_bounds(&bytes);
        bytes
    }
    #[inline]
    fn check_bounds(bytes: &[u8]) {
        if let Bound::Bounded {
            max_size,
            is_fixed_size,
        } = Self::BOUND
        {
            let actual = bytes.len();
            if is_fixed_size {
                assert_eq!(
                    actual, max_size as usize,
                    "expected a fixed-size element with length {max_size} bytes, but found {actual} bytes"
                );
            } else {
                assert!(
                    actual <= max_size as usize,
                    "expected an element with length <= {max_size} bytes, but found {actual} bytes"
                );
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Bound {
    Unbounded,
    Bounded { max_size: u32, is_fixed_size: bool },
}

impl Bound {
    pub const fn max_size(&self) -> u32 {
        if let Self::Bounded { max_size, .. } = self {
            *max_size
        } else {
            panic!("Cannot get max size of unbounded type.");
        }
    }
    pub const fn is_fixed_size(&self) -> bool {
        if let Self::Bounded { is_fixed_size, .. } = self {
            *is_fixed_size
        } else {
            false
        }
    }
}

pub struct Bounds {
    pub max_size: u32,
    pub is_fixed_size: bool,
}
pub const fn bounds<A: Storable>() -> Bounds {
    if let Bound::Bounded {
        max_size,
        is_fixed_size,
    } = A::BOUND
    {
        Bounds {
            max_size,
            is_fixed_size,
        }
    } else {
        panic!("Cannot get bounds of unbounded type.");
    }
}

pub const fn bytes_to_store_size_bounded(bounds: &Bounds) -> u32 {
    if bounds.is_fixed_size {
        0
    } else {
        bytes_to_store_size(bounds.max_size)
    }
}

const fn bytes_to_store_size(bytes_size: u32) -> u32 {
    if bytes_size <= 0xff {
        1
    } else if bytes_size <= 0xffff {
        2
    } else {
        4
    }
}
