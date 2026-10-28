//! Read-only adapter for the documented ic-stable-structures 0.7.2 MGR V1
//! layout. No ledger or payload reads. Keep the dependency pinned when changing
//! this adapter; layout changes require a new review, not a fallback decoder.
use super::RuntimeConstructionError;
use crate::WASM_PAGE_SIZE_BYTES;
use ic_platform::stable_structures::Memory;

pub(super) const IDS: usize = 255;
pub(super) const BUCKET_CAPACITY: u16 = 32_768;
pub(super) const BUCKETS: usize = BUCKET_CAPACITY as usize;
pub(super) const HEADER_BYTES: usize = 40 + IDS * 8;
pub(super) const METADATA_BYTES: usize = HEADER_BYTES + BUCKETS;

///
/// MemoryManagerLayoutError
///
/// Invalid or unsupported persisted manager metadata. No writes are performed
/// when reporting these failures. Memory implementations must obey [`Memory`].
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MemoryManagerLayoutError {
    /// This adapter supports the little-endian IC/current native layout only.
    #[error("unsupported big-endian manager layout")]
    UnsupportedByteOrder,
    /// The persisted bucket size is zero.
    #[error("persisted bucket size is zero")]
    ZeroBucketSize,
    /// Reserved header fields are not recognized by this adapter.
    #[error("nonzero reserved manager header bytes")]
    ReservedHeader,
    /// The allocated count exceeds the finite bucket table.
    #[error("allocated bucket count {count} exceeds 32768")]
    BucketCount { count: u16 },
    /// Allocated buckets must be a dense prefix and all later entries unused.
    #[error("invalid bucket table entry at index {index}")]
    BucketTable { index: u16 },
    /// Virtual extent disagrees with the number of assigned buckets.
    #[error("virtual extent and bucket count disagree for memory ID {id}")]
    VirtualExtent { id: u8 },
    /// Backing memory does not cover every assigned bucket.
    #[error("physical extent {physical_pages} pages is below assigned end {required_pages}")]
    TruncatedBacking {
        physical_pages: u64,
        required_pages: u64,
    },
    /// Backing size cannot be represented in bytes.
    #[error("backing memory extent overflows bytes")]
    ExtentOverflow,
    /// A live manager's cached state differs from persisted metadata.
    #[error("persisted manager metadata differs from runtime authority")]
    RuntimeMismatch,
}

pub(super) struct Layout {
    pub metadata: [u8; METADATA_BYTES],
    pub physical_pages: u64,
    pub bucket_pages: u16,
    pub allocated_buckets: u16,
    pub pages: [u64; IDS],
    pub buckets: [u16; IDS],
}

// Exact expected metadata, never refreshed from foreign mutations. The pinned
// manager is still the only writer. Growth projects its known header/table
// edits into this read-only authority before comparing the persisted result.
pub(super) struct ManagerAuthority {
    expected: Box<[u8]>,
    poisoned: std::cell::Cell<bool>,
}

impl ManagerAuthority {
    pub fn capture(memory: &impl Memory) -> Self {
        let mut expected = vec![0; METADATA_BYTES].into_boxed_slice();
        memory.read(0, &mut expected);
        Self {
            expected,
            poisoned: std::cell::Cell::new(false),
        }
    }

    pub fn allocated_buckets(&self) -> u16 {
        u16::from_le_bytes([self.expected[4], self.expected[5]])
    }

    pub fn verify(&self, memory: &impl Memory) -> Result<(), MemoryManagerLayoutError> {
        self.check_healthy()?;
        #[expect(
            clippy::large_stack_arrays,
            reason = "fixed manager metadata bound, no payload reads or heap allocation"
        )]
        let mut actual = [0; METADATA_BYTES];
        let required_pages = 1 + u64::from(self.allocated_buckets())
            * u64::from(u16::from_le_bytes([self.expected[6], self.expected[7]]));
        if memory.size() < required_pages {
            self.poisoned.set(true);
            return Err(MemoryManagerLayoutError::RuntimeMismatch);
        }
        memory.read(0, &mut actual);
        self.verify_observed(&actual)
    }

    pub const fn check_healthy(&self) -> Result<(), MemoryManagerLayoutError> {
        if self.poisoned.get() {
            Err(MemoryManagerLayoutError::RuntimeMismatch)
        } else {
            Ok(())
        }
    }

    pub fn poison(&self) {
        self.poisoned.set(true);
    }

    pub fn verify_observed(&self, actual: &[u8]) -> Result<(), MemoryManagerLayoutError> {
        // MGR V1's complete 34848-byte image is an exact number of words.
        // Explicit word comparison avoids Wasm's byte-at-a-time memcmp while
        // preserving every byte of the admission boundary, including unused IDs.
        #[cfg(target_arch = "wasm32")]
        let matches = {
            let (actual_words, actual_tail) = actual.as_chunks::<8>();
            let (expected_words, expected_tail) = self.expected.as_chunks::<8>();
            actual.len() == METADATA_BYTES
                && self.expected.len() == METADATA_BYTES
                && actual_tail.is_empty()
                && expected_tail.is_empty()
                && actual_words
                    .iter()
                    .zip(expected_words)
                    .all(|(actual, expected)| {
                        u64::from_le_bytes(*actual) == u64::from_le_bytes(*expected)
                    })
        };
        // Native slice equality uses the optimized host comparator, including
        // in debug test builds. Both paths compare the complete exact image.
        #[cfg(not(target_arch = "wasm32"))]
        let matches = actual.len() == METADATA_BYTES && actual == self.expected.as_ref();
        if self.poisoned.get() || !matches {
            self.poisoned.set(true);
            return Err(MemoryManagerLayoutError::RuntimeMismatch);
        }
        Ok(())
    }

    pub fn record_growth(&mut self, id: u8, pages: u64, allocated: u16) {
        let old = usize::from(self.allocated_buckets());
        self.expected[HEADER_BYTES + old..HEADER_BYTES + usize::from(allocated)].fill(id);
        self.expected[4..6].copy_from_slice(&allocated.to_le_bytes());
        let offset = 40 + usize::from(id) * 8;
        self.expected[offset..offset + 8].copy_from_slice(&pages.to_le_bytes());
    }
}

pub(super) fn read<M: Memory>(memory: &M) -> Result<Layout, RuntimeConstructionError> {
    if cfg!(target_endian = "big") {
        return Err(MemoryManagerLayoutError::UnsupportedByteOrder.into());
    }
    let physical_pages = memory.size();
    physical_pages
        .checked_mul(WASM_PAGE_SIZE_BYTES)
        .ok_or(MemoryManagerLayoutError::ExtentOverflow)?;
    // The caller handles empty memory. Even one page covers all metadata.
    if physical_pages == 0 {
        return Err(MemoryManagerLayoutError::TruncatedBacking {
            physical_pages,
            required_pages: 1,
        }
        .into());
    }
    #[expect(
        clippy::large_stack_arrays,
        reason = "bounded manager image is also the diagnostic authority observation"
    )]
    let mut metadata = [0; METADATA_BYTES];
    let (header, table) = metadata.split_at_mut(HEADER_BYTES);
    memory.read(0, header);
    let observed_magic = [header[0], header[1], header[2]];
    if observed_magic != *b"MGR" {
        return Err(RuntimeConstructionError::ForeignMemory { observed_magic });
    }
    if header[3] != 1 {
        return Err(RuntimeConstructionError::UnsupportedMemoryManagerVersion {
            observed: header[3],
            supported: 1,
        });
    }
    if header[8..40].iter().any(|byte| *byte != 0) {
        return Err(MemoryManagerLayoutError::ReservedHeader.into());
    }
    let allocated_buckets = u16::from_le_bytes([header[4], header[5]]);
    let bucket_pages = u16::from_le_bytes([header[6], header[7]]);
    if bucket_pages == 0 {
        return Err(MemoryManagerLayoutError::ZeroBucketSize.into());
    }
    if usize::from(allocated_buckets) > BUCKETS {
        return Err(MemoryManagerLayoutError::BucketCount {
            count: allocated_buckets,
        }
        .into());
    }
    let required_pages = 1 + u64::from(allocated_buckets) * u64::from(bucket_pages);
    if physical_pages < required_pages {
        return Err(MemoryManagerLayoutError::TruncatedBacking {
            physical_pages,
            required_pages,
        }
        .into());
    }
    // Fixed local buffer and read bound, independent of physical size/history.
    memory.read(HEADER_BYTES as u64, table);
    let mut buckets = [0_u16; IDS];
    for (index, id) in (0_u16..32_768).zip(table.iter().copied()) {
        if (index < allocated_buckets) != (id != 255) {
            return Err(MemoryManagerLayoutError::BucketTable { index }.into());
        }
        if id != 255 {
            buckets[usize::from(id)] += 1;
        }
    }
    let mut pages = [0; IDS];
    for (id, value) in (0_u8..255).zip(&mut pages) {
        let offset = 40 + usize::from(id) * 8;
        *value = u64::from_le_bytes(header[offset..offset + 8].try_into().expect("eight bytes"));
        if value.div_ceil(u64::from(bucket_pages)) != u64::from(buckets[usize::from(id)]) {
            return Err(MemoryManagerLayoutError::VirtualExtent { id }.into());
        }
    }
    Ok(Layout {
        metadata,
        physical_pages,
        bucket_pages,
        allocated_buckets,
        pages,
        buckets,
    })
}
