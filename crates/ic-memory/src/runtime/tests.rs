use super::{
    MemoryRuntime, RuntimeConstructionError, RuntimeDiagnosticError, RuntimeOpenError,
    fixture_diagnostics::diagnostic_validation_ledger,
    fixtures::{declarations, pool},
    policy::GenericAllocationPolicy,
};
use crate::{
    DiagnosticMemorySize, LedgerCommitError, MemoryManagerSlot, PolicyIdentity,
    RuntimeBootstrapPolicy, SealedDeclarationSnapshot, StableKey,
    testing::{DiagnosticCheck, DiagnosticCode},
};
use ic_platform::stable_structures::{Cell, Memory, VectorMemory};
use std::convert::Infallible;

fn empty_runtime() -> MemoryRuntime<VectorMemory> {
    MemoryRuntime::new(VectorMemory::default()).expect("empty backing memory")
}

const WASM_PAGE_SIZE_BYTES: usize = 65_536;

fn one_page_backing(prefix: [u8; 4]) -> (VectorMemory, Vec<u8>) {
    let memory = VectorMemory::default();
    assert_eq!(memory.grow(1), 0);
    let mut bytes = vec![0xA5; WASM_PAGE_SIZE_BYTES];
    bytes[..prefix.len()].copy_from_slice(&prefix);
    memory.write(0, &bytes);
    (memory, bytes)
}

fn read_one_page(memory: &VectorMemory) -> Vec<u8> {
    assert_eq!(memory.size(), 1);
    let mut bytes = vec![0; WASM_PAGE_SIZE_BYTES];
    memory.read(0, &mut bytes);
    bytes
}

struct CountingPolicy(std::cell::Cell<usize>);

impl RuntimeBootstrapPolicy for CountingPolicy {
    type Error = Infallible;

    fn validate_allocation(
        &self,
        key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        assert_ne!(key.as_str(), crate::IC_MEMORY_LEDGER_STABLE_KEY);
        self.0.set(self.0.get() + 1);
        Ok(())
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"runtime-tests.counting-policy...")
    }
}

struct IdentityPolicy(PolicyIdentity);

impl IdentityPolicy {
    const fn new(bytes: [u8; 32]) -> Self {
        Self(PolicyIdentity::new(bytes))
    }
}

impl RuntimeBootstrapPolicy for IdentityPolicy {
    type Error = Infallible;

    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        self.0
    }
}

struct RejectPolicy;

impl RuntimeBootstrapPolicy for RejectPolicy {
    type Error = &'static str;

    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Err("rejected")
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"runtime-tests.reject-policy.....")
    }
}

mod bootstrap;
mod construction;
mod diagnostics;
