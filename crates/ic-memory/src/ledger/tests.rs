//! Shared ledger fixtures and contract-specific regression modules.

use super::*;
use crate::{CommitRecoveryError, test_wire::hex_fixture};

use crate::{
    declaration::AllocationDeclaration, key::StableKey, physical::CommittedGenerationBytes,
    requests::DeclarationSnapshot, slot::MemoryManagerSlot,
};
use ic_platform::stable_structures::Storable;
fn declaration(key: &str, id: u8) -> AllocationDeclaration {
    AllocationDeclaration::new(key, MemoryManagerSlot::new(id).expect("usable slot"))
        .expect("declaration")
}

fn ledger() -> AllocationLedger {
    AllocationLedger {
        current_generation: 3,
        records: Vec::new(),
    }
}

fn committed_ledger(current_generation: u64) -> AllocationLedger {
    AllocationLedger {
        current_generation,
        records: Vec::new(),
    }
}

fn allocation_record(key: &str, id: u8) -> AllocationRecord {
    AllocationRecord::from_declaration(&declaration(key, id))
}

fn snapshot_from(declarations: Vec<AllocationDeclaration>) -> DeclarationSnapshot {
    DeclarationSnapshot::for_test(declarations)
}

fn stage(
    ledger: AllocationLedger,
    snapshot: &DeclarationSnapshot,
) -> Result<AllocationLedger, crate::RuntimeBootstrapError<std::convert::Infallible>> {
    stage_generation(
        RecoveredLedger::from_trusted_ledger(ledger),
        snapshot_from(snapshot.declarations().to_vec()),
        &crate::GenericAllocationPolicy,
    )
    .map(|(ledger, _)| ledger)
}

fn record<'ledger>(ledger: &'ledger AllocationLedger, key: &str) -> &'ledger AllocationRecord {
    ledger
        .records()
        .iter()
        .find(|record| record.stable_key.as_str() == key)
        .expect("allocation record")
}

fn encoded_payload(ledger: &AllocationLedger) -> Vec<u8> {
    codec::encode_ledger(ledger).expect("fixture ledger")
}

fn ledger_from_payload_fixture(contents: &str, generation: u64) -> AllocationLedger {
    let bytes = hex_fixture(contents);
    let ledger = decode_ledger(&bytes, generation).expect("fixture ledger");
    ledger
        .validate_integrity()
        .expect("fixture ledger integrity");
    assert_eq!(bytes, encoded_payload(&ledger));
    ledger
}

fn store_from_fixture(contents: &str) -> LedgerCommitStore {
    let bytes = hex_fixture(contents);
    let store = crate::decode_stable_cell_commit_store(&bytes).expect("fixture store");
    assert_eq!(
        bytes,
        crate::stable_cell::EncodedLedgerRecord::new(&store)
            .unwrap()
            .into_bytes()
    );
    store
}

fn committed_fixture_ledger() -> AllocationLedger {
    AllocationLedger {
        current_generation: 1,
        records: vec![allocation_record("app.users.v1", 100)],
    }
}

mod encoding;
mod recovery;
mod transitions;
