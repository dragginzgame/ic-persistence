//! Recover, admit and persist allocation bindings before publishing authority.
use super::{
    BootstrapAdmission, CommittedRuntime, MemoryRuntime, RuntimeBootstrapBinding,
    RuntimeBootstrapError, RuntimeLifecycle, RuntimeMemory,
};
use crate::{
    LedgerCommitStore, PolicyIdentity, RuntimeBootstrapPolicy, STABLE_CELL_VALUE_OFFSET,
    SealedDeclarationSnapshot,
    slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::{EncodedLedgerRecord, decode_stable_cell_commit_store_from_memory},
};
use ic_platform::stable_structures::{Cell, Memory};

impl<M: Memory> MemoryRuntime<M> {
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

    pub(super) fn validate_pool_custody(
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
}

impl RuntimeBootstrapBinding {
    pub(super) fn validate<P>(
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
