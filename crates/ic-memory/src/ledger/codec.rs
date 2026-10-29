//! The checked physical header owns generation; Wire owns bounded record bytes.

use crate::{constants::MAX_ALLOCATIONS, encoding::LEDGER_LIMITS};
use serde::Deserialize;

pub(super) fn encode_ledger(
    ledger: &super::AllocationLedger,
) -> Result<Vec<u8>, super::LedgerIntegrityError> {
    ic_wire::native::encode(ledger.records(), LEDGER_LIMITS).map_err(|error| match error {
        ic_wire::Error::Limit => super::LedgerIntegrityError::LimitExceeded {
            resource: "ledger encoding",
            limit: crate::constants::MAX_LEDGER_BYTES,
        },
        other => panic!("allocation ledger serialization failed: {other}"),
    })
}

pub(super) fn decode_ledger(
    bytes: &[u8],
    generation: u64,
) -> Result<super::AllocationLedger, String> {
    // Borrow all untrusted text: no String allocation precedes key admission.
    #[derive(Deserialize)]
    struct Record<'a> {
        stable_key: &'a str,
        slot: u8,
    }
    let records: ic_wire::List<Record<'_>, MAX_ALLOCATIONS> =
        ic_wire::native::decode(bytes, LEDGER_LIMITS).map_err(|error| error.to_string())?;
    let records = records
        .into_vec()
        .into_iter()
        .map(|record| {
            Ok(super::AllocationRecord {
                stable_key: crate::StableKey::parse(record.stable_key)
                    .map_err(|error| error.to_string())?,
                slot: crate::MemoryManagerSlot::new(record.slot)
                    .map_err(|error| error.to_string())?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(super::AllocationLedger {
        current_generation: generation,
        records,
    })
}
