//! Explicit fixture preflight; production hosts own operator report assembly.

use super::{MemoryRuntime, RuntimeLifecycle};
use crate::{
    DiagnosticExport, DiagnosticMemorySize, LedgerCommitError, PolicyIdentity, RecoveredLedger,
    RuntimeBootstrapPolicy, SealedDeclarationSnapshot, physical::CommitSnapshotDiagnostic,
    slot::MEMORY_MANAGER_LEDGER_ID, stable_cell::decode_stable_cell_commit_store_from_memory,
};
use ic_platform::stable_structures::{Memory, memory_manager::MemoryId};
use serde::Serialize;
use std::{borrow::Cow, fmt::Display};

///
/// DiagnosticRuntimeBinding
///
/// Fixture view of the policy identity and declaration snapshot bound
/// to one successful memory-runtime bootstrap.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticRuntimeBinding {
    /// Bounded semantic identity of the bootstrap policy.
    pub policy_identity: PolicyIdentity,
    /// Exact host namespace grants and physical exclusions bound at bootstrap.
    pub allocation_pool: crate::MemoryAllocationPool,
}

impl DiagnosticRuntimeBinding {
    /// Build one runtime binding diagnostic.
    #[must_use]
    pub const fn new(
        policy_identity: PolicyIdentity,
        allocation_pool: crate::MemoryAllocationPool,
    ) -> Self {
        Self {
            policy_identity,
            allocation_pool,
        }
    }
}

///
/// MemoryRuntimeDoctorReport
///
/// Preflight and runtime diagnostic report for one concrete
/// [`crate::MemoryRuntime`].
///
/// This report is intended for explicitly enabled fixture diagnostics. Recoverable
/// runtime problems, such as corrupt stable-cell bytes or commit recovery
/// failure, are represented as fields instead of aborting report construction.
/// Reports support serialization only; applications own diagnostic input protocols.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryRuntimeDoctorReport {
    /// Whether this runtime has completed bootstrap validation.
    pub bootstrapped: bool,
    /// Policy identity supplied for this diagnostic evaluation.
    pub tested_policy_identity: PolicyIdentity,
    /// Binding published by this runtime's successful bootstrap, when present.
    pub established_bootstrap_binding: Option<DiagnosticRuntimeBinding>,
    /// Whether the tested identity and declarations match the established
    /// bootstrap binding.
    pub bootstrap_binding: DiagnosticCheck,
    /// Stable-cell ledger storage status.
    pub stable_cell: DiagnosticStableCell,
    /// Protected commit recovery status when a ledger record was readable.
    pub commit_recovery: Option<CommitSnapshotDiagnostic>,
    /// Recovered allocation ledger export when protected recovery succeeded.
    pub ledger: Option<DiagnosticExport>,
    /// Key-only requirements contributed by linked components.
    pub requests: Vec<crate::MemoryRequest>,
    /// Tested host namespace grants and physical exclusions.
    pub allocation_pool: crate::MemoryAllocationPool,
    /// Declaration validation result under the tested caller-supplied policy.
    pub validation: DiagnosticCheck,
}

///
/// DiagnosticCode
///
/// Stable machine-readable category for an operator diagnostic failure.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DiagnosticCode {
    /// Stable-cell storage could not be decoded.
    #[serde(rename = "stable_cell")]
    StableCell,
    /// Persisted bytes use a recognized but unsupported durable format.
    #[serde(rename = "unsupported_format")]
    UnsupportedFormat,
    /// Protected ledger recovery failed.
    #[serde(rename = "ledger_recovery")]
    LedgerRecovery,
    /// Current declarations failed allocation validation.
    #[serde(rename = "allocation_validation")]
    AllocationValidation,
    /// Tested bootstrap identity or declarations differ from runtime state.
    #[serde(rename = "runtime_binding")]
    RuntimeBinding,
}

///
/// DiagnosticFailure
///
/// Machine-readable diagnostic code paired with an operator-facing message.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticFailure {
    /// Stable diagnostic category.
    pub code: DiagnosticCode,
    /// Human-readable failure detail.
    pub message: String,
}

impl DiagnosticFailure {
    /// Build a coded diagnostic failure.
    #[must_use]
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

///
/// DiagnosticStableCell
///
/// Read-only diagnostic view of the stable-cell ledger storage envelope.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticStableCell {
    /// Stable-cell status.
    pub status: DiagnosticStableCellStatus,
    /// Backing memory size for the ledger cell.
    pub memory_size: DiagnosticMemorySize,
}

impl DiagnosticStableCell {
    /// Build a stable-cell diagnostic.
    #[must_use]
    pub const fn new(
        status: DiagnosticStableCellStatus,
        memory_size: DiagnosticMemorySize,
    ) -> Self {
        Self {
            status,
            memory_size,
        }
    }
}

///
/// DiagnosticStableCellStatus
///
/// Stable-cell ledger storage status.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum DiagnosticStableCellStatus {
    /// The ledger memory is empty and can be initialized.
    Empty,
    /// The stable-cell envelope and ledger record decoded successfully.
    Readable,
    /// The ledger memory is present but could not be decoded as the expected
    /// stable-cell ledger record.
    Corrupt {
        /// Stable-cell envelope or ledger-record decode failure.
        failure: DiagnosticFailure,
    },
}

///
/// DiagnosticCheck
///
/// Read-only diagnostic status for a preflight check.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum DiagnosticCheck {
    /// The check could not run because prerequisite state was unavailable.
    NotRun {
        /// Stable diagnostic category.
        code: DiagnosticCode,
        /// Reason the check could not run.
        message: String,
    },
    /// The check completed successfully.
    Passed,
    /// The check ran and found a problem.
    Failed {
        /// Stable diagnostic category.
        code: DiagnosticCode,
        /// Validation failure.
        message: String,
    },
}

impl DiagnosticCheck {
    /// Build a passed diagnostic check.
    #[must_use]
    pub const fn passed() -> Self {
        Self::Passed
    }

    /// Build a failed diagnostic check.
    #[must_use]
    pub fn failed(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::Failed {
            code,
            message: message.into(),
        }
    }

    /// Build a skipped diagnostic check.
    #[must_use]
    pub fn not_run(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::NotRun {
            code,
            message: message.into(),
        }
    }
}

/// Build preflight and lifecycle diagnostics for this runtime.
///
/// Validation checks the supplied declarations and allocation policy only.
/// It does not execute `prepare_bootstrap`, predict its completed set, or
/// certify consumer admission. Diagnostics never replay preparation.
#[must_use]
pub fn doctor_report<M: Memory, P>(
    runtime: &MemoryRuntime<M>,
    declarations: &SealedDeclarationSnapshot,
    pool: &crate::MemoryAllocationPool,
    policy: &P,
) -> MemoryRuntimeDoctorReport
where
    P: RuntimeBootstrapPolicy,
    P::Error: Display,
{
    let observation = runtime.ledger_observation();
    let tested_policy_identity = policy.runtime_bootstrap_identity();
    let established_bootstrap_binding = runtime.established_bootstrap_binding();
    let bootstrap_binding = diagnostic_bootstrap_binding(
        &tested_policy_identity,
        declarations,
        pool,
        &runtime.lifecycle,
    );
    let validation = diagnostic_validation(
        runtime,
        declarations,
        pool,
        policy,
        observation
            .recovery
            .as_ref()
            .map(|(recovered, _)| recovered),
    );
    let (stable_cell, commit_recovery, ledger) = observation.into_parts();

    MemoryRuntimeDoctorReport {
        bootstrapped: runtime.is_bootstrapped(),
        tested_policy_identity,
        established_bootstrap_binding,
        bootstrap_binding,
        stable_cell,
        commit_recovery,
        ledger,
        requests: declarations.requests().to_vec(),
        allocation_pool: pool.clone(),
        validation,
    }
}

// One read-only observation owns storage/recovery status and memory readings
// taken before policy callbacks. It never carries open or allocation authority.
struct LedgerObservation {
    stable_cell: DiagnosticStableCell,
    recovery: Option<(
        Result<RecoveredLedger, LedgerCommitError>,
        CommitSnapshotDiagnostic,
    )>,
    observed_pages: [u64; crate::constants::MAX_ALLOCATIONS],
}

impl LedgerObservation {
    fn into_parts(
        self,
    ) -> (
        DiagnosticStableCell,
        Option<CommitSnapshotDiagnostic>,
        Option<DiagnosticExport>,
    ) {
        let commit_recovery = self.recovery.as_ref().map(|(_, diagnostic)| *diagnostic);
        let ledger = self.recovery.and_then(|(recovered, _)| {
            recovered.ok().map(|recovered| {
                DiagnosticExport::from_owned_ledger(recovered.into_ledger()).with_memory_sizes(
                    |record| self.observed_pages[usize::from(record.slot().id())],
                )
            })
        });
        (self.stable_cell, commit_recovery, ledger)
    }
}

impl<M: Memory> MemoryRuntime<M> {
    fn established_bootstrap_binding(&self) -> Option<DiagnosticRuntimeBinding> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => None,
            RuntimeLifecycle::Bootstrapped { binding, .. } => Some(DiagnosticRuntimeBinding::new(
                binding.policy_identity,
                binding.pool.clone(),
            )),
        }
    }

    fn ledger_observation(&self) -> LedgerObservation {
        let memory = self
            .memory_manager
            .get(MemoryId::new(MEMORY_MANAGER_LEDGER_ID));
        let memory_size = DiagnosticMemorySize::from_wasm_pages(memory.size());
        let record = decode_stable_cell_commit_store_from_memory(&memory);
        drop(memory);
        let status = match &record {
            Ok(_) if memory_size.wasm_pages == 0 => DiagnosticStableCellStatus::Empty,
            Ok(_) => DiagnosticStableCellStatus::Readable,
            Err(error) => DiagnosticStableCellStatus::Corrupt {
                failure: DiagnosticFailure::new(DiagnosticCode::StableCell, error.to_string()),
            },
        };
        // Release decoded physical bytes before measuring slots or invoking policy.
        let recovery = record.ok().map(|record| record.recover_with_diagnostic());
        let mut observed_pages = [0; crate::constants::MAX_ALLOCATIONS];
        if let Some((Ok(recovered), _)) = &recovery {
            for record in recovered.ledger().records() {
                let id = record.slot().id();
                observed_pages[usize::from(id)] = self.memory_size_pages(id);
            }
        }
        LedgerObservation {
            stable_cell: DiagnosticStableCell::new(status, memory_size),
            recovery,
            observed_pages,
        }
    }
}

fn diagnostic_validation<M: Memory, P: RuntimeBootstrapPolicy>(
    runtime: &MemoryRuntime<M>,
    declarations: &SealedDeclarationSnapshot,
    pool: &crate::MemoryAllocationPool,
    custom_policy: &P,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> DiagnosticCheck
where
    P::Error: Display,
{
    let recovered = match diagnostic_validation_ledger(recovered) {
        Ok(recovered) => recovered,
        Err(failure) => return DiagnosticCheck::not_run(failure.code, failure.message),
    };
    let occupied = recovered.ledger().occupied_slots();
    if let Err(error) = runtime.validate_pool_custody(&occupied, pool) {
        return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, error.to_string());
    }
    let resolved = match declarations.resolve(&recovered, &occupied, Vec::new(), pool) {
        Ok(resolved) => resolved,
        Err(err) => {
            return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string());
        }
    };
    match crate::validation::check_allocations(&recovered, &resolved, custom_policy) {
        Ok(()) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string()),
    }
}

fn diagnostic_bootstrap_binding(
    tested_policy_identity: &PolicyIdentity,
    declarations: &SealedDeclarationSnapshot,
    pool: &crate::MemoryAllocationPool,
    lifecycle: &RuntimeLifecycle,
) -> DiagnosticCheck {
    let RuntimeLifecycle::Bootstrapped { binding, .. } = lifecycle else {
        return DiagnosticCheck::not_run(
            DiagnosticCode::RuntimeBinding,
            "runtime has not completed bootstrap",
        );
    };
    match binding.validate::<std::convert::Infallible>(declarations, pool, tested_policy_identity) {
        Ok(()) => DiagnosticCheck::passed(),
        Err(_) => DiagnosticCheck::failed(
            DiagnosticCode::RuntimeBinding,
            "tested policy/declaration/pool binding differs from established runtime binding",
        ),
    }
}

pub(super) fn diagnostic_validation_ledger(
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> Result<Cow<'_, crate::RecoveredLedger>, DiagnosticFailure> {
    if let Some(Ok(recovered)) = recovered {
        return Ok(Cow::Borrowed(recovered));
    }
    if let Some(Err(err)) = recovered {
        if let Some(genesis) = RecoveredLedger::genesis_for_absent_snapshot(err) {
            return Ok(Cow::Owned(genesis));
        }
        let code = if matches!(
            err,
            LedgerCommitError::Recovery(crate::CommitRecoveryError::UnsupportedFormat { .. })
        ) {
            DiagnosticCode::UnsupportedFormat
        } else {
            DiagnosticCode::LedgerRecovery
        };
        return Err(DiagnosticFailure::new(
            code,
            format!("protected ledger recovery: {err}"),
        ));
    }
    Err(DiagnosticFailure::new(
        DiagnosticCode::StableCell,
        "stable-cell ledger record is not readable",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_outcome_states_serialize() {
        let stable_cell = DiagnosticStableCell::new(
            DiagnosticStableCellStatus::Corrupt {
                failure: DiagnosticFailure::new(
                    DiagnosticCode::StableCell,
                    "bad stable-cell record",
                ),
            },
            DiagnosticMemorySize::from_wasm_pages(1),
        );
        let allocation_pool = crate::MemoryAllocationPool::new(Vec::new(), Vec::new()).unwrap();
        let check = DiagnosticCheck::failed(
            DiagnosticCode::AllocationValidation,
            "duplicate declaration",
        );

        assert_eq!(
            serde_json::to_value(DiagnosticCheck::passed()).unwrap(),
            "Passed"
        );
        assert_eq!(
            serde_json::to_value(check).unwrap(),
            serde_json::json!({
                "Failed": {"code": "allocation_validation", "message": "duplicate declaration"}
            })
        );
        assert_eq!(
            serde_json::to_value(stable_cell).unwrap(),
            serde_json::json!({
                "status": {"Corrupt": {"failure": {"code": "stable_cell", "message": "bad stable-cell record"}}},
                "memory_size": {"wasm_pages": 1}
            })
        );
        assert_eq!(
            serde_json::to_value(allocation_pool).unwrap(),
            serde_json::json!({
                "authorities": [], "excluded_ids": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
            })
        );
    }

    #[test]
    fn diagnostic_codes_have_stable_wire_names() {
        let cases = [
            (DiagnosticCode::StableCell, "stable_cell"),
            (DiagnosticCode::UnsupportedFormat, "unsupported_format"),
            (DiagnosticCode::LedgerRecovery, "ledger_recovery"),
            (
                DiagnosticCode::AllocationValidation,
                "allocation_validation",
            ),
            (DiagnosticCode::RuntimeBinding, "runtime_binding"),
        ];

        for (code, expected) in cases {
            assert_eq!(
                serde_json::to_value(code).expect("diagnostic code value"),
                serde_json::Value::String(expected.to_string())
            );
        }
    }
}
