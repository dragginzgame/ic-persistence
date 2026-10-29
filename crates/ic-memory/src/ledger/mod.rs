mod claim;
mod codec;
mod error;
mod integrity;
mod record;
mod stage;

#[cfg(any(test, feature = "test-support"))]
use crate::physical::CommitSnapshotDiagnostic;
use crate::physical::CommittedGenerationBytes;
use serde::{Deserialize, Serialize};

pub use claim::{ClaimConflict, ClaimOutcome, validate_declaration_claim};
pub use error::{AllocationStageError, LedgerCommitError, LedgerIntegrityError};
pub use record::{AllocationLedger, AllocationRecord, RecoveredLedger};
pub use stage::stage_generation;

use codec::decode_ledger;

///
/// LedgerCommitStore
///
/// Generation-scoped allocation ledger commit store.
///
/// This type owns the logical commit lifecycle and the crate-owned IC Wire ledger
/// encoding. It deliberately does not own stable-memory IO; that remains the
/// substrate or framework owner's responsibility.
///
/// The runtime commits allocation ledger generations through this passive DTO.
/// Decoding it grants no memory access or persistence authority. It does not
/// open stable-memory handles or allocate application slots.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerCommitStore {
    /// Protected current physical snapshot.
    #[serde(deserialize_with = "crate::encoding::deserialize_present_option")]
    pub(crate) current: Option<CommittedGenerationBytes>,
}

impl LedgerCommitStore {
    /// Recover the authoritative allocation ledger using the native IC Wire ledger codec.
    pub(crate) fn recover(&self) -> Result<RecoveredLedger, LedgerCommitError> {
        let committed = self.authoritative().map_err(LedgerCommitError::Recovery)?;
        Self::recover_committed(committed)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn recover_with_diagnostic(
        &self,
    ) -> (
        Result<RecoveredLedger, LedgerCommitError>,
        CommitSnapshotDiagnostic,
    ) {
        let (committed, diagnostic) = self.authoritative_with_diagnostic();
        let recovered = committed
            .map_err(LedgerCommitError::Recovery)
            .and_then(Self::recover_committed);
        (recovered, diagnostic)
    }

    fn recover_committed(
        committed: &CommittedGenerationBytes,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        let ledger = decode_ledger(committed.payload(), committed.generation())
            .map_err(LedgerCommitError::Codec)?;
        ledger
            .validate_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        Ok(RecoveredLedger::from_trusted_ledger(ledger))
    }

    /// Recover the authoritative ledger, or use logical genesis for an empty store.
    ///
    /// Only explicit snapshot absence admits the empty generation-zero base.
    /// No payload is encoded or committed here. Corrupt or partially written
    /// stores fail closed; the caller must persist its snapshot successor
    /// before publishing allocation authority.
    pub(crate) fn recover_or_genesis(&self) -> Result<RecoveredLedger, LedgerCommitError> {
        self.recover()
            .or_else(|error| RecoveredLedger::genesis_for_absent_snapshot(&error).ok_or(error))
    }

    /// Commit one logical allocation ledger generation through the native IC Wire ledger codec.
    ///
    /// # Panics
    ///
    /// Panics only if the private concrete ledger-encoding invariant is broken.
    #[cfg(test)]
    pub fn commit(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        self.commit_generation(ledger)?;
        // The current writer encoded this integrity-checked ledger, and the
        // physical commit checked its predecessor and established this generation.
        // Existing persisted bytes still cross the full recovery boundary.
        Ok(RecoveredLedger::from_trusted_ledger(ledger.clone()))
    }

    // Bootstrap already owns its staged ledger. Share the checked mutation
    // without cloning that ledger into a proof only to unwrap it immediately.
    pub(crate) fn commit_generation(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCommitError> {
        ledger
            .validate_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        let payload = codec::encode_ledger(ledger).map_err(LedgerCommitError::Integrity)?;
        self.commit_payload_at_generation(ledger.current_generation, payload)
            .map(|_| ())
            .map_err(LedgerCommitError::Recovery)
    }

    /// Simulate corruption of the current logical ledger snapshot.
    #[cfg(test)]
    pub fn write_corrupt_ledger(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCommitError> {
        let payload = codec::encode_ledger(ledger).map_err(LedgerCommitError::Integrity)?;
        self.write_corrupt_snapshot(ledger.current_generation, payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
