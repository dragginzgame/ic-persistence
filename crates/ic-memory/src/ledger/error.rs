use crate::{key::StableKey, physical::CommitRecoveryError, slot::MemoryManagerSlot};

///
/// LedgerIntegrityError
///
/// Decoded ledger violates current ownership invariants.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerIntegrityError {
    /// Current structural recovery ceiling exceeded.
    #[error("ledger exceeds {resource} limit {limit}")]
    LimitExceeded {
        resource: &'static str,
        limit: usize,
    },
    /// Stable key appears in more than one retained allocation record.
    #[error("stable key '{stable_key}' appears in more than one allocation record")]
    DuplicateStableKey { stable_key: StableKey },
    /// Memory ID appears in more than one retained allocation record.
    #[error("allocation slot '{slot:?}' appears in more than one allocation record")]
    DuplicateSlot { slot: MemoryManagerSlot },
    /// Commit counter zero is reserved for empty genesis.
    #[error("genesis ledger contains allocation records")]
    NonemptyGenesis,
}

///
/// LedgerDecodeError
///
/// Typed source of a logical ledger decoding failure. Decoded rows still
/// require ledger integrity validation before granting allocation authority.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerDecodeError {
    /// IC Wire refused syntax, trailing bytes or traversal budgets.
    #[error(transparent)]
    Wire(#[from] ic_wire::Error),
    /// A stored logical key violates its bounded grammar.
    #[error(transparent)]
    Key(#[from] crate::StableKeyError),
    /// A stored slot is outside the usable manager ID domain.
    #[error(transparent)]
    Slot(#[from] crate::MemoryManagerSlotError),
}

///
/// LedgerCommitError
///
/// Failure to recover or commit a logical allocation ledger.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerCommitError {
    /// Protected physical commit recovery failed.
    #[error(transparent)]
    Recovery(CommitRecoveryError),
    /// Built-in ledger decoding failed.
    #[error("allocation ledger codec failed: {0}")]
    Codec(#[source] LedgerDecodeError),
    /// Decoded ledger violates current ownership invariants.
    #[error(transparent)]
    Integrity(LedgerIntegrityError),
}

///
/// AllocationStageError
///
/// Failure to stage a validated allocation generation.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationStageError {
    /// The commit counter cannot advance without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow { generation: u64 },
    #[error(transparent)]
    Integrity(#[from] LedgerIntegrityError),
}
