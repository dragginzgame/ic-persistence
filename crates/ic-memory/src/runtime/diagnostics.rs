use super::{MemoryRuntime, RuntimeDiagnosticError};
use crate::{DiagnosticExport, RecoveredLedger, physical::CommitSnapshotDiagnostic};
use ic_platform::stable_structures::Memory;

impl<M: Memory> MemoryRuntime<M> {
    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        self.growth
            .verify()
            .map_err(super::RuntimeConstructionError::from)?;
        let recovered = self.ledger_record_from_memory()?.recover()?;
        Ok(self.recovered_diagnostic_export(recovered))
    }

    /// Diagnose protected commit recovery from this runtime's ledger memory.
    ///
    /// This operation is available before bootstrap when the stable-cell
    /// envelope is readable or the ledger memory is empty.
    pub fn commit_recovery_diagnostic(
        &self,
    ) -> Result<CommitSnapshotDiagnostic, RuntimeDiagnosticError> {
        self.growth
            .verify()
            .map_err(super::RuntimeConstructionError::from)?;
        let record = self.ledger_record_from_memory()?;
        Ok(record.diagnostic())
    }

    fn recovered_diagnostic_export(&self, recovered: RecoveredLedger) -> DiagnosticExport {
        DiagnosticExport::from_owned_ledger(recovered.into_ledger())
            .with_memory_sizes(|record| self.memory_size_pages(record.slot().id()))
    }
}
