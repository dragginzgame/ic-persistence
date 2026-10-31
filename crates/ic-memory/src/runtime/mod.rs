mod admission;
#[cfg(test)]
mod admission_tests;
mod adoption;
#[cfg(test)]
mod adoption_tests;
mod allocations;
mod backing;
mod bootstrap;
mod config;
mod construction;
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
    LedgerCommitStore, PolicyIdentity, StableCellLedgerError, requests::SealedDeclarationSnapshot,
    slot::MEMORY_MANAGER_LEDGER_ID, stable_cell::decode_stable_cell_commit_store_from_memory,
};
use ic_platform::stable_structures::{
    Memory,
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
    /// Return whether this runtime has published committed allocation authority.
    #[must_use]
    pub const fn is_bootstrapped(&self) -> bool {
        matches!(self.lifecycle, RuntimeLifecycle::Bootstrapped(_))
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
