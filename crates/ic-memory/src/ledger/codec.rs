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
) -> Result<super::AllocationLedger, super::LedgerDecodeError> {
    // Borrow all untrusted text: no String allocation precedes key admission.
    let records: ic_wire::List<Record<'_>, MAX_ALLOCATIONS> =
        ic_wire::native::decode(bytes, LEDGER_LIMITS)?;
    let records = records
        .into_vec()
        .into_iter()
        .map(|record| {
            Ok(super::AllocationRecord {
                stable_key: crate::StableKey::parse(record.stable_key)?,
                slot: crate::MemoryManagerSlot::new(record.slot)?,
            })
        })
        .collect::<Result<Vec<_>, super::LedgerDecodeError>>()?;
    Ok(super::AllocationLedger {
        current_generation: generation,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoding_preserves_wire_key_and_slot_error_sources() {
        for (rows, expected) in [
            (vec![("invalid", 0_u8)], "key"),
            (vec![("app.store.v1", u8::MAX)], "slot"),
        ] {
            let bytes = ic_wire::native::encode(&rows, LEDGER_LIMITS).unwrap();
            let error = decode_ledger(&bytes, 1).unwrap_err();
            match (expected, &error) {
                ("key", crate::LedgerDecodeError::Key(_))
                | ("slot", crate::LedgerDecodeError::Slot(_)) => {}
                _ => panic!("wrong typed failure: {error:?}"),
            }
            let error = crate::LedgerCommitError::Codec(error);
            assert!(std::error::Error::source(&error).is_some());
        }
        let bytes = ic_wire::native::encode(&vec![("app.store.v1", 0_u8)], LEDGER_LIMITS).unwrap();
        let mut trailing = bytes;
        trailing.push(0);
        assert_eq!(
            decode_ledger(&trailing, 1).unwrap_err(),
            crate::LedgerDecodeError::Wire(ic_wire::Error::Trailing)
        );
    }
}
