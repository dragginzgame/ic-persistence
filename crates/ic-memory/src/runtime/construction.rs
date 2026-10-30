//! Backing custody and manager construction before bootstrap authority exists.
use super::{
    MemoryBacking, MemoryManagerConfig, MemoryManagerLayoutError, MemoryRuntime,
    RuntimeConstructionError, RuntimeGrowError, RuntimeLifecycle, backing, custody, layout,
};
use ic_platform::stable_structures::{Memory, memory_manager::MemoryManager};
use std::rc::Rc;

impl<M: Memory> MemoryRuntime<M> {
    /// Construct an unbootstrapped runtime without overwriting foreign memory.
    ///
    /// Empty backing memory is initialized as an
    /// [`MemoryManager`]. Nonempty memory must pass bounded
    /// validation of the current manager header, bucket table, and extents; otherwise
    /// construction returns a typed error before the manager can
    /// write its header or allocation table. A pre-grown blank memory is
    /// nonempty and is therefore rejected rather than assumed disposable.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeConstructionError::BackingAlreadyClaimed`] before any
    /// backing IO if another runtime or handle retains custody.
    /// Returns [`RuntimeConstructionError::ForeignMemory`] for nonempty memory
    /// without `MemoryManager` magic, or
    /// [`RuntimeConstructionError::UnsupportedMemoryManagerVersion`] when the
    /// magic is recognized but the layout version is not current. Invalid
    /// metadata returns [`RuntimeConstructionError::Layout`]. Refused fresh
    /// metadata growth returns [`RuntimeConstructionError::Growth`] without
    /// writes, allowing the same backing memory to be retried. Reopening honors
    /// the actual persisted bucket size; only fresh memory uses 128 pages.
    pub fn new(memory: M) -> Result<Self, RuntimeConstructionError>
    where
        M: MemoryBacking,
    {
        Self::construct(memory, None)
    }

    /// Construct with an explicit immutable bucket policy. Existing memory must
    /// match exactly; mismatches fail before manager initialization or writes.
    ///
    /// # Errors
    ///
    /// Returns the construction errors described by [`Self::new`], or
    /// [`RuntimeConstructionError::BucketSizeMismatch`] when existing geometry
    /// differs from `config`.
    pub fn new_with_config(
        memory: M,
        config: MemoryManagerConfig,
    ) -> Result<Self, RuntimeConstructionError>
    where
        M: MemoryBacking,
    {
        Self::construct(memory, Some(config))
    }

    fn construct(
        memory: M,
        requested: Option<MemoryManagerConfig>,
    ) -> Result<Self, RuntimeConstructionError>
    where
        M: MemoryBacking,
    {
        // Locals drop in reverse order. Establish backing ownership before its
        // claim so refusal/unwind also retires the identity before freeing it.
        let backing = Rc::new(memory);
        let claim = custody::BackingClaim::acquire(backing.as_ref())?;
        if cfg!(target_endian = "big") {
            return Err(MemoryManagerLayoutError::UnsupportedByteOrder.into());
        }
        let bucket_size_pages = if backing.size() == 0 {
            // Reserve the metadata page before the dependency writes its header.
            // This fresh allocation is owned here; caller-supplied nonempty
            // memory still passes the layout checks below before any writes.
            if backing.grow(1) == -1 {
                return Err(RuntimeGrowError::BackingRefused {
                    additional_pages: 1,
                }
                .into());
            }
            requested.unwrap_or_default().bucket_size_pages()
        } else {
            let measured = layout::read(backing.as_ref())?;
            let actual = measured.bucket_pages;
            if let Some(config) = requested {
                check_bucket_size(actual, config)?;
            }
            actual
        };
        let memory_manager =
            MemoryManager::init_with_bucket_size(Rc::clone(&backing), bucket_size_pages);
        let authority =
            std::cell::RefCell::new(layout::ManagerAuthority::capture(backing.as_ref()));
        let growth = Rc::new(backing::GrowthState {
            _claim: claim,
            backing: Rc::clone(&backing),
            bucket_size_pages,
            authority,
            open_slots: [const { std::cell::Cell::new(0) }; 4],
        });
        Ok(Self {
            memory_manager,
            growth,
            lifecycle: RuntimeLifecycle::Unbootstrapped,
        })
    }

    /// Return the immutable bucket configuration bound to this runtime's manager.
    #[must_use]
    pub fn memory_manager_config(&self) -> MemoryManagerConfig {
        // Construction has already validated the nonzero persisted setting.
        MemoryManagerConfig::from_validated(self.growth.bucket_size_pages)
    }
}

const fn check_bucket_size(
    actual: u16,
    requested: MemoryManagerConfig,
) -> Result<(), RuntimeConstructionError> {
    if actual != requested.bucket_size_pages() {
        return Err(RuntimeConstructionError::BucketSizeMismatch {
            persisted: actual,
            requested: requested.bucket_size_pages(),
        });
    }
    Ok(())
}
