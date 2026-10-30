use super::{AllocationLedger, AllocationRecord};
use crate::declaration::AllocationDeclaration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimOutcome {
    Existing,
    New,
}

///
/// ClaimConflict
///
/// Internal claim failure carrying the historical record needed for error
/// projection. Existing bindings never require ledger mutation.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimConflict<'ledger> {
    StableKeyMoved { record: &'ledger AllocationRecord },
    SlotReused { record: &'ledger AllocationRecord },
}

pub fn validate_declaration_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    declaration: &AllocationDeclaration,
) -> Result<ClaimOutcome, ClaimConflict<'ledger>> {
    if let Some(record) = ledger.record_for_key(&declaration.stable_key) {
        if record.slot != declaration.slot {
            return Err(ClaimConflict::StableKeyMoved { record });
        }
        return Ok(ClaimOutcome::Existing);
    }

    if let Some(record) = ledger
        .records()
        .iter()
        .find(|record| record.slot == declaration.slot)
    {
        return Err(ClaimConflict::SlotReused { record });
    }
    Ok(ClaimOutcome::New)
}
