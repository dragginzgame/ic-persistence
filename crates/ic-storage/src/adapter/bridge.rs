//! Private codec and memory views for the copied collection engines.
use super::Storable;
use crate::{Memory, engine};
use std::borrow::Cow;

pub(super) struct Backing<M>(pub M);
impl<M: Memory> engine::Memory for Backing<M> {
    fn size(&self) -> u64 {
        self.0.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        self.0.grow(pages)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) {
        self.0.read(offset, bytes);
    }
    fn write(&self, offset: u64, bytes: &[u8]) {
        self.0.write(offset, bytes);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub(super) struct Codec<T>(pub T);
impl<T> Codec<T> {
    #[expect(
        unsafe_code,
        reason = "Transparent immutable codec view preserves key identity without a clone"
    )]
    pub(super) const fn borrowed(value: &T) -> &Self {
        // SAFETY: repr(transparent) gives identical layout/alignment and the
        // wrapper adds no invariant. This shared view has the input lifetime.
        unsafe { &*std::ptr::from_ref(value).cast::<Self>() }
    }
}
impl<T: Storable> engine::Storable for Codec<T> {
    const BOUND: engine::storable::Bound = match T::BOUND {
        super::Bound::Unbounded => engine::storable::Bound::Unbounded,
        super::Bound::Bounded {
            max_size,
            is_fixed_size,
        } => engine::storable::Bound::Bounded {
            max_size,
            is_fixed_size,
        },
    };
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.0
            .to_bytes_checked()
            .expect("collection encoding failed")
    }
    fn into_bytes(self) -> Vec<u8> {
        self.0
            .into_bytes_checked()
            .expect("collection encoding failed")
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        T::check_bounds(&bytes).expect("stored collection value exceeds codec bounds");
        Self(T::from_bytes(bytes).expect("stored collection value failed to decode"))
    }
}

pub(super) type Map<K, V, M> = engine::BTreeMap<Codec<K>, Codec<V>, Backing<M>>;

pub(super) fn admit_map<M: Memory>(memory: &M) -> super::error::Result<()> {
    use super::error::{CollectionError as Error, ensure};
    ensure(memory.size() != 0, Error::Format)?;
    let mut header = [0; 12];
    memory.read(0, &mut header);
    ensure(
        &header[..4] == b"BTR\x02" && header[8..12] == u32::MAX.to_le_bytes(),
        Error::Format,
    )
}

pub(super) fn reserve<M: Memory>(memory: &M) -> super::error::Result<()> {
    if memory.size() == 0 && memory.grow(1) == -1 {
        return Err(super::CollectionError::Growth);
    }
    Ok(())
}
