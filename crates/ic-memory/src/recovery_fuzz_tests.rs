//! Reproducible bounded mutation campaigns over the maintained recovery boundaries.
use crate::{Memory, VectorMemory, stable_cell, test_cbor::hex_fixture};

fn exercise(bytes: &[u8]) {
    let memory = VectorMemory::default();
    assert_eq!(memory.grow(2), 0);
    let mut header = *b"SCL\x01\0\0\0\0";
    header[4..].copy_from_slice(&u32::try_from(bytes.len()).unwrap().to_le_bytes());
    memory.write(0, &header);
    memory.write(8, bytes);
    let before = memory.borrow().clone();
    if let Ok(store) = stable_cell::decode_stable_cell_commit_store_from_memory(&memory)
        && let Ok(recovered) = store.recover()
    {
        recovered.ledger().validate_integrity().unwrap();
    }
    assert_eq!(*memory.borrow(), before, "recovery must never write");
    // Recompute the physical envelope so mutations also reach logical decoding,
    // rather than stopping every payload mutation at the outer checksum.
    let mut store = crate::LedgerCommitStore::default();
    if store
        .commit_payload_at_generation(1, bytes.to_vec())
        .is_ok()
        && let Ok(recovered) = store.recover()
    {
        recovered.ledger().validate_integrity().unwrap();
    }
}

fn campaign(mut seed: u64, iterations: usize) {
    let corpus = [
        hex_fixture(include_str!(
            "../fixtures/current/ledger_commit_store_single.cbor.hex"
        )),
        hex_fixture(include_str!(
            "../fixtures/current/single_allocation_ledger.cbor.hex"
        )),
        vec![0x9f, 0xff],
        vec![0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        vec![0x81; 34],
    ];
    for bytes in &corpus {
        exercise(bytes);
        for end in 0..bytes.len() {
            exercise(&bytes[..end]);
        }
    }
    for iteration in 0..iterations {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let mut bytes = corpus[iteration % corpus.len()].clone();
        let position = usize::try_from(seed % bytes.len() as u64).unwrap();
        match iteration % 4 {
            0 => bytes[position] ^= seed.to_le_bytes()[4],
            1 => bytes.truncate(position),
            2 => bytes.extend_from_slice(&seed.to_le_bytes()),
            _ => bytes
                .splice(position..=position, seed.to_le_bytes())
                .for_each(drop),
        }
        exercise(&bytes);
    }
}

#[test]
fn recovery_mutation_smoke() {
    campaign(0x1C50, 256);
}

#[test]
#[ignore = "explicit bounded mutation campaign; optional IC_MEMORY_FUZZ_SEED reproduces a run"]
fn recovery_mutation_campaign() {
    let seed = std::env::var("IC_MEMORY_FUZZ_SEED")
        .map_or(Ok(0x1C50), |value| value.parse::<u64>())
        .unwrap();
    println!("recovery mutation seed={seed}, iterations=10000");
    campaign(seed, 10_000);
}
