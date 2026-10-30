mod admission;
#[cfg(test)]
mod admission_tests;
mod adoption;
#[cfg(test)]
mod adoption_tests;
mod allocations;
mod backing;
mod config;
mod custody;
mod diagnostics;
mod error;
#[cfg(any(test, feature = "test-support"))]
pub mod fixture_diagnostics;
#[cfg(test)]
mod fixtures;
mod layout;
mod policy;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod allocation_tests;
#[cfg(test)]
mod custody_tests;
#[cfg(test)]
mod growth_tests;
#[cfg(test)]
#[expect(
    unsafe_code,
    reason = "exercise raw reads with valid uninitialized destinations"
)]
mod read_tests;
#[cfg(test)]
mod request_tests;
#[cfg(test)]
mod tests;

pub use admission::{BootstrapAdmission, BootstrapAdmissionError};
pub use adoption::RuntimeAdoptionError;

pub use allocations::{
    MemoryAllocation, MemoryAllocationSummary, MemoryAllocations, MemoryBindingSummary,
};
pub use backing::RuntimeMemory;
pub use config::MemoryManagerConfig;
pub use custody::{BackingIdentity, MemoryBacking};
pub use error::{
    MemoryResolutionError, RuntimeBootstrapError, RuntimeConstructionError, RuntimeDiagnosticError,
    RuntimeGrowError, RuntimeOpenError,
};
pub use layout::MemoryManagerLayoutError;
pub use policy::GenericAllocationPolicy;

use crate::{
    LedgerCommitStore, PolicyIdentity, RuntimeBootstrapPolicy, STABLE_CELL_VALUE_OFFSET,
    StableCellLedgerError,
    requests::SealedDeclarationSnapshot,
    slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::{EncodedLedgerRecord, decode_stable_cell_commit_store_from_memory},
};
use ic_platform::stable_structures::{
    Cell, Memory,
    memory_manager::{MemoryId, MemoryManager},
};

use std::rc::Rc;

enum RuntimeLifecycle {
    Unbootstrapped,
    Bootstrapped(CommittedRuntime),
}

struct CommittedRuntime {
    allocations: Vec<crate::AllocationDeclaration>,
    generation: u64,
    binding: RuntimeBootstrapBinding,
}

struct RuntimeBootstrapBinding {
    source: SealedDeclarationSnapshot,
    policy_identity: PolicyIdentity,
    pool: crate::MemoryAllocationPool,
}

///
/// MemoryRuntime
///
/// Canonical owner of allocation bootstrap state for one backing memory.
///
/// The runtime owns its `MemoryManager`, allocation-ledger persistence, bootstrap
/// lifecycle, committed allocation capability, opens, and diagnostics. Static
/// linked-program declarations are supplied separately as one immutable
/// [`SealedDeclarationSnapshot`].
///
/// Each bootstrap attempt fallibly decodes its ledger record from persisted
/// memory. Capacity-checked writes use `Cell`; diagnostics also read persisted
/// memory directly.
///
/// Construction requires [`MemoryBacking`], which identifies physical aliases.
/// The backing need not be `Send`, `Sync`, `Clone`, or `'static`.
///
/// A backing claim rejects another runtime before backing IO while this runtime
/// or any detached lease lives. Drop them all before constructing a replacement.
/// Standard VectorMemory/default canister backings identify aliases automatically;
/// custom hosts must uphold [`MemoryBacking`]'s identity contract. A composition
/// selects one IC Memory crate identity and shares one runtime. Raw aliased IO or
/// unmanaged managers remain prohibited. Bounded metadata checks refuse stale
/// manager state; they are not transactions against arbitrary concurrent raw IO.
///

pub struct MemoryRuntime<M: Memory> {
    memory_manager: MemoryManager<Rc<M>>,
    // Shared backing, immutable geometry and live accounting belong to growth.
    // Handles reserve physical capacity; the manager owns bucket metadata.
    growth: Rc<backing::GrowthState<M>>,
    lifecycle: RuntimeLifecycle,
}

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

    /// Return whether this runtime has published committed allocation authority.
    #[must_use]
    pub const fn is_bootstrapped(&self) -> bool {
        matches!(self.lifecycle, RuntimeLifecycle::Bootstrapped(_))
    }

    /// Bootstrap this backing memory from one immutable declaration snapshot.
    ///
    /// Recovery, metadata admission, logical resolution, policy evaluation,
    /// staging, persistence and binding publication are local to this runtime.
    /// A repeated call is idempotent only when the sealed declaration snapshot,
    /// canonical host pool and [`RuntimeBootstrapPolicy::runtime_bootstrap_identity`] match the
    /// successful bootstrap. A mismatch returns a typed error without
    /// advancing the durable generation or re-evaluating policy.
    /// Independently sealed snapshots match when their canonical contents are equal.
    ///
    /// # Panics
    ///
    /// Panics if a private runtime or encoding invariant is broken, or backing
    /// memory or a policy callback panics.
    pub fn bootstrap<P: RuntimeBootstrapPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        pool: &crate::MemoryAllocationPool,
        policy: &P,
    ) -> Result<u64, RuntimeBootstrapError<P::Error>> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => self.bootstrap_unbootstrapped(
                declarations,
                pool,
                policy,
                policy.runtime_bootstrap_identity(),
            ),
            RuntimeLifecycle::Bootstrapped(_) => self.verify_bootstrap(declarations, pool, policy),
        }
    }

    /// Verify the exact host bootstrap binding through a shared runtime reference.
    ///
    /// Returns the established generation only when declarations, pool and the
    /// policy identity match. Evaluates the policy identity once, but never runs
    /// admission or allocation validation, reads backing memory, writes, grows,
    /// or opens an allocation. An unbootstrapped runtime is refused rather than
    /// initialized. Hosts may share one committed runtime through `Rc` without
    /// reconstructing its binding checks or attempting mutable bootstrap again.
    pub fn verify_bootstrap<P: RuntimeBootstrapPolicy>(
        &self,
        declarations: &SealedDeclarationSnapshot,
        pool: &crate::MemoryAllocationPool,
        policy: &P,
    ) -> Result<u64, RuntimeBootstrapError<P::Error>> {
        let committed = self
            .committed()
            .map_err(|_| RuntimeBootstrapError::NotBootstrapped)?;
        committed
            .binding
            .validate(declarations, pool, &policy.runtime_bootstrap_identity())?;
        Ok(committed.generation)
    }

    const fn committed_state(&self) -> Option<&CommittedRuntime> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => None,
            RuntimeLifecycle::Bootstrapped(committed) => Some(committed),
        }
    }

    fn committed(&self) -> Result<&CommittedRuntime, RuntimeOpenError> {
        self.committed_state()
            .ok_or(RuntimeOpenError::NotBootstrapped)
    }

    fn validate_pool_custody(
        &self,
        occupied: &[bool; crate::constants::MAX_ALLOCATIONS],
        pool: &crate::MemoryAllocationPool,
    ) -> Result<(), super::MemoryResolutionError> {
        for id in 0..crate::MEMORY_MANAGER_INVALID_ID {
            if pool.contains(id) && self.memory_size_pages(id) != 0 && !occupied[usize::from(id)] {
                return Err(super::MemoryResolutionError::UnmanagedAllocation { id });
            }
        }
        Ok(())
    }

    fn bootstrap_unbootstrapped<P: RuntimeBootstrapPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        pool: &crate::MemoryAllocationPool,
        policy: &P,
        policy_identity: PolicyIdentity,
    ) -> Result<u64, RuntimeBootstrapError<P::Error>> {
        self.growth.verify()?;
        let mut memory = self
            .memory(MEMORY_MANAGER_LEDGER_ID)
            .expect("governance memory is not publicly leasable");
        let mut record = decode_stable_cell_commit_store_from_memory(&memory)?;
        let recovered = record.recover_or_genesis()?;
        let occupied = recovered.ledger().occupied_slots();
        self.validate_pool_custody(&occupied, pool)?;
        if memory.size() == 0 {
            // Empty memory decodes to an uninitialized record. Preserve fresh
            // cell acquisition before admission without persisting genesis.
            memory = write_ledger_cell(memory, &LedgerCommitStore::default())?;
        }
        let mut admission = BootstrapAdmission::new(recovered.ledger(), declarations, pool);
        let preparation = policy.prepare_bootstrap(&mut admission);
        let historical = admission.complete()?;
        preparation.map_err(RuntimeBootstrapError::AdmissionPolicy)?;
        let resolved = declarations.resolve(&recovered, &occupied, historical, pool)?;
        let (generation, resolved) = {
            let (staged, resolved) = crate::ledger::stage_generation(recovered, resolved, policy)?;
            record.commit_generation(&staged)?;
            (staged.current_generation(), resolved)
        };
        drop(write_ledger_cell(memory, &record)?);
        // Publish application bindings only after the durable Cell write succeeds.
        let mut allocations = resolved;
        allocations.retain(|declaration| {
            !crate::is_ic_memory_stable_key(declaration.stable_key().as_str())
        });
        self.lifecycle = RuntimeLifecycle::Bootstrapped(CommittedRuntime {
            allocations,
            generation,
            binding: RuntimeBootstrapBinding {
                source: declarations.clone(),
                policy_identity,
                pool: pool.clone(),
            },
        });
        Ok(generation)
    }

    /// Borrow this runtime's persisted application bindings without IO.
    /// These passive views cannot open memory independently of the runtime.
    pub fn committed_allocations(
        &self,
    ) -> Result<&[crate::AllocationDeclaration], RuntimeOpenError> {
        Ok(&self.committed()?.allocations)
    }

    /// Return the in-memory persisted generation without decoding the ledger.
    pub const fn committed_generation(&self) -> Result<u64, RuntimeOpenError> {
        match self.committed_state() {
            Some(committed) => Ok(committed.generation),
            None => Err(RuntimeOpenError::NotBootstrapped),
        }
    }

    /// Borrow the host pool bound by successful bootstrap for host projections.
    /// This is current policy, never evidence of historical ownership or free IDs.
    pub const fn allocation_pool(&self) -> Result<&crate::MemoryAllocationPool, RuntimeOpenError> {
        match self.committed_state() {
            Some(committed) => Ok(&committed.binding.pool),
            None => Err(RuntimeOpenError::NotBootstrapped),
        }
    }

    /// Lease by durable key using this runtime's persisted current bindings.
    ///
    /// A second live open returns [`RuntimeOpenError::AlreadyOpen`] without IO.
    /// Dropping the handle, or a collection that owns it, permits reopening.
    /// A successful new lease validates the complete persisted manager image;
    /// detected divergence poisons all of this runtime's existing handles.
    pub fn open_memory(&self, stable_key: &str) -> Result<RuntimeMemory<M>, RuntimeOpenError> {
        self.memory(self.memory_id(stable_key)?)
    }

    /// Resolve an application key's committed ID without opening memory,
    /// reading history, or changing the host's policy or bucket configuration.
    pub fn memory_id(&self, stable_key: &str) -> Result<u8, RuntimeOpenError> {
        crate::key::validate(stable_key)?;
        if crate::is_ic_memory_stable_key(stable_key) {
            return Err(RuntimeOpenError::ReservedStableKey {
                stable_key: stable_key.to_string(),
            });
        }
        let declaration =
            crate::declaration::declaration_for_key(self.committed_allocations()?, stable_key)
                .ok_or_else(|| RuntimeOpenError::StableKeyNotCommitted(stable_key.to_string()))?;
        Ok(declaration.slot().id())
    }

    fn memory(&self, id: u8) -> Result<RuntimeMemory<M>, RuntimeOpenError> {
        let (word, mask) = self.growth.lease_bit(id);
        if word.get() & mask != 0 {
            return Err(RuntimeOpenError::AlreadyOpen { id });
        }
        self.growth.verify()?;
        let memory = self.memory_manager.get(MemoryId::new(id));
        word.set(word.get() | mask);
        Ok(RuntimeMemory {
            memory,
            growth: Rc::clone(&self.growth),
            id,
        })
    }

    // Size-only observations need no handle carrying runtime growth authority.
    fn memory_size_pages(&self, id: u8) -> u64 {
        self.memory_manager.get(MemoryId::new(id)).size()
    }

    fn ledger_record_from_memory(&self) -> Result<LedgerCommitStore, StableCellLedgerError> {
        // Decoding only reads; the manager handle retains backing lifetime
        // without carrying the growth owner needed by writable runtime handles.
        let memory = self
            .memory_manager
            .get(MemoryId::new(MEMORY_MANAGER_LEDGER_ID));
        decode_stable_cell_commit_store_from_memory(&memory)
    }
}

impl RuntimeBootstrapBinding {
    fn validate<P>(
        &self,
        declarations: &SealedDeclarationSnapshot,
        pool: &crate::MemoryAllocationPool,
        policy_identity: &PolicyIdentity,
    ) -> Result<(), RuntimeBootstrapError<P>> {
        if &self.source != declarations {
            return Err(RuntimeBootstrapError::DeclarationSnapshotMismatch);
        }
        if &self.pool != pool {
            return Err(RuntimeBootstrapError::AllocationPoolMismatch);
        }
        if &self.policy_identity != policy_identity {
            return Err(RuntimeBootstrapError::PolicyIdentityMismatch {
                established: self.policy_identity,
                requested: *policy_identity,
            });
        }
        Ok(())
    }
}

fn write_ledger_cell<M: Memory, P>(
    memory: RuntimeMemory<M>,
    record: &LedgerCommitStore,
) -> Result<RuntimeMemory<M>, RuntimeBootstrapError<P>> {
    let encoded =
        EncodedLedgerRecord::new(record).map_err(RuntimeBootstrapError::StableCellLedgerEncode)?;
    let value_size = encoded.len();
    // The record limit is below u32::MAX, including the eight-byte cell header.
    let required_bytes = STABLE_CELL_VALUE_OFFSET + value_size as u64;
    let available_bytes = memory.size().saturating_mul(crate::WASM_PAGE_SIZE_BYTES);
    if required_bytes > available_bytes {
        let grow_by = (required_bytes - available_bytes).div_ceil(crate::WASM_PAGE_SIZE_BYTES);
        memory.grow(grow_by)?;
    }
    Ok(Cell::new(memory, encoded).into_memory())
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
