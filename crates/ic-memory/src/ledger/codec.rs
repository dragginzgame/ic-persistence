//! The checked physical header owns generation; Wire owns bounded record bytes.

use crate::{constants::MAX_ALLOCATIONS, encoding::LEDGER_LIMITS};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize, Serialize)]
struct Record<'a> {
    stable_key: &'a str,
    slot: u8,
}

struct Records<'a>(&'a [super::AllocationRecord]);

impl Serialize for Records<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(|record| Record {
            stable_key: record.stable_key().as_str(),
            slot: record.slot().id(),
        }))
    }
}

pub(super) fn encode_ledger(
    ledger: &super::AllocationLedger,
) -> Result<Vec<u8>, super::LedgerIntegrityError> {
    ic_wire::native::encode(&Records(ledger.records()), LEDGER_LIMITS).map_err(
        |error| match error {
            ic_wire::Error::Limit => super::LedgerIntegrityError::LimitExceeded {
                resource: "ledger encoding",
                limit: crate::constants::MAX_LEDGER_BYTES,
            },
            other => panic!("allocation ledger serialization failed: {other}"),
        },
    )
}

pub(super) fn decode_ledger(
    bytes: &[u8],
    generation: u64,
) -> Result<super::AllocationLedger, String> {
    // Borrow all untrusted text: no String allocation precedes key admission.
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
