//! Installed collection/lease qualification; no application codec or recovery policy.

#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
use ic_storage_test_fixtures::storage as protocol;

#[cfg(target_arch = "wasm32")]
#[expect(
    unsafe_code,
    reason = "fixture-only raw IC entrypoints and reply calls"
)]
mod canister {
    use super::protocol;
    use ic_storage::adapter::{
        BTreeMapRegion, CollectionError, StableBTreeMap, StableBTreeSet, StableCell, StableVec,
    };
    use ic_storage::{
        DefaultMemoryImpl, MemoryRuntime, StableCellPayloadError, StorageConfig,
        testing::fixture_runtime,
    };
    use ic_wire::native;
    use serde::{Serialize, de::DeserializeOwned};
    use std::{cell::RefCell, rc::Rc};

    thread_local! {
        static RUNTIME: RefCell<Option<MemoryRuntime<DefaultMemoryImpl>>> = const { RefCell::new(None) };
    }

    #[link(wasm_import_module = "ic0")]
    unsafe extern "C" {
        fn msg_arg_data_size() -> usize;
        fn msg_arg_data_copy(dst: usize, offset: usize, size: usize);
        fn msg_reply_data_append(src: usize, size: usize);
        fn msg_reply();
    }

    fn argument<T: DeserializeOwned>(id: u64) -> T {
        // SAFETY: Admit the complete frame before allocating/copying bytes.
        let bytes = unsafe {
            let size = msg_arg_data_size();
            assert!(size <= protocol::REQUEST.bytes);
            let mut bytes = vec![0; size];
            msg_arg_data_copy(bytes.as_mut_ptr() as usize, 0, size);
            bytes
        };
        native::decode_request(id, &bytes, protocol::REQUEST).unwrap()
    }

    fn reply(id: u64, value: &impl Serialize) {
        let bytes = native::encode_response(id, value, protocol::RESPONSE).unwrap();
        // SAFETY: The encoded source is live through the synchronous system copy.
        unsafe {
            msg_reply_data_append(bytes.as_ptr() as usize, bytes.len());
            msg_reply();
        }
    }

    fn with_runtime<T>(operation: impl FnOnce(&MemoryRuntime<DefaultMemoryImpl>) -> T) -> T {
        RUNTIME.with(|runtime| operation(runtime.borrow().as_ref().unwrap()))
    }

    fn bootstrap() {
        let mut keys = vec![
            "storage.map.v1".to_owned(),
            "storage.cell.v1".to_owned(),
            "storage.region.v1".to_owned(),
            "storage.journal.v1".to_owned(),
            "storage.probe.v1".to_owned(),
            "storage.set.v1".to_owned(),
            "storage.vector.v1".to_owned(),
        ];
        keys.extend((0..1024).map(|index| format!("storage.extra.n{index:04}.v1")));
        let names: Vec<_> = keys.iter().map(String::as_str).collect();
        let runtime = fixture_runtime(
            DefaultMemoryImpl::default(),
            "storage",
            "storage.",
            &names,
            StorageConfig::default(),
        )
        .unwrap();
        let high_key = ic_storage::StableKey::parse("storage.extra.n1023.v1").unwrap();
        assert!(runtime.allocation_id(&high_key).unwrap().get() > 255);
        let report = runtime.diagnostics().unwrap();
        assert_eq!(report.allocations, names.len());
        assert_eq!(report.admitted_allocations, names.len());
        assert_eq!(report.live_leases, 0);
        assert_eq!(report.generation, runtime.committed_generation().unwrap());
        assert_eq!(
            report.physical_pages,
            ic_storage::Memory::size(&DefaultMemoryImpl::default())
        );
        assert_eq!(
            report.physical_pages,
            1 + report.metadata_pages + report.data_pages + report.unassigned_pages
        );
        RUNTIME.with(|slot| *slot.borrow_mut() = Some(runtime));
    }

    fn region(runtime: &MemoryRuntime<DefaultMemoryImpl>) -> BTreeMapRegion<u64, u64> {
        BTreeMapRegion::new(
            Rc::new(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.region.v1").unwrap())
                    .unwrap(),
            ),
            0..64,
        )
    }

    #[unsafe(export_name = "canister_init")]
    extern "C" fn init() {
        argument::<()>(protocol::INIT_ID);
        bootstrap();
        with_runtime(|runtime| {
            let mut map = StableBTreeMap::<u64, u64>::create(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.map.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            let mut set = StableBTreeSet::<u64>::create(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.set.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            set.insert(10).unwrap();
            let vector = StableVec::<u64>::create(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.vector.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            vector.push(&10).unwrap();
            let _ = StableCell::create(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.extra.n1023.v1").unwrap())
                    .unwrap(),
                1023_u64,
            )
            .unwrap();
            // Prepare capacity once so admission-failure probes perform no allocation.
            runtime
                .open_memory(&ic_storage::StableKey::parse("storage.probe.v1").unwrap())
                .unwrap()
                .grow(1)
                .unwrap();
            let mut view = region(runtime);
            view.initialize().unwrap();
            for key in 0..128 {
                map.insert(key, key + 10).unwrap();
                view.insert(key, key + 20).unwrap();
            }
            let _ = StableCell::create(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.cell.v1").unwrap())
                    .unwrap(),
                30_u64,
            )
            .unwrap();
            let _ = StableCell::create(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.journal.v1").unwrap())
                    .unwrap(),
                30_u64,
            )
            .unwrap();
        });
    }

    fn replace(value: u64) {
        with_runtime(|runtime| {
            let mut map = StableBTreeMap::<u64, u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.map.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            let mut set = StableBTreeSet::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.set.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            map.set_node_cache_budget(16 * 1024);
            // Warm routing before the mutation exercised by update/rollback.
            let _ = map.get(&10).unwrap();
            let previous = set.iter().next().unwrap().unwrap();
            assert!(set.remove(&previous).unwrap());
            set.insert(value).unwrap();
            let vector = StableVec::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.vector.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            vector.set(0, &value).unwrap();
            map.insert(0, value).unwrap();
            region(runtime).insert(0, value + 1).unwrap();
            StableCell::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.cell.v1").unwrap())
                    .unwrap(),
                16 * 1024 * 1024,
            )
            .unwrap()
            .set(value + 2)
            .unwrap();
            StableCell::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.journal.v1").unwrap())
                    .unwrap(),
                16 * 1024 * 1024,
            )
            .unwrap()
            .set(value + 2)
            .unwrap();
            assert!(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.map.v1").unwrap())
                    .is_err()
            );
        });
    }

    #[unsafe(export_name = "canister_update mutate")]
    extern "C" fn mutate() {
        let mode: u8 = argument(protocol::MUTATE_ID);
        replace(100);
        assert_eq!(mode, 0, "injected trap after collection writes");
        reply(protocol::MUTATE_ID, &());
    }

    #[unsafe(export_name = "canister_update cell_load_failure")]
    extern "C" fn cell_load_failure() {
        let mode: u8 = argument(protocol::CELL_FAILURE_ID);
        with_runtime(|runtime| {
            let memory = runtime
                .open_memory(&ic_storage::StableKey::parse("storage.probe.v1").unwrap())
                .unwrap();
            if mode == 4 {
                drop(StableCell::reset(memory, vec![7_u8]).unwrap());
                let memory = runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.probe.v1").unwrap())
                    .unwrap();
                // Hosts may deliberately trap on a typed codec refusal; IC rollback
                // must restore both bytes and the exclusive lease in this case.
                let _ =
                    StableCell::<u64>::load(memory, 8).expect("host chose rollback on codec error");
                unreachable!();
            }
            if mode != 0 {
                if memory.size() == 0 {
                    memory.grow(1).unwrap();
                }
                let mut header = [0; ic_storage::STABLE_CELL_HEADER_SIZE];
                header[..8].copy_from_slice(b"ICCELL01");
                match mode {
                    1 => header[..8].copy_from_slice(b"FOREIGN!"),
                    2 => header[12] = 1,
                    3 => header[8..12].copy_from_slice(&u32::MAX.to_le_bytes()),
                    _ => panic!("unknown probe mode"),
                }
                memory.write(0, &header);
            }
            let pages = memory.size();
            let mut before = vec![0; usize::try_from(pages * 65_536).unwrap()];
            memory.read(0, &mut before);
            let error = StableCell::<u64>::load(memory, 16 * 1024 * 1024)
                .err()
                .unwrap();
            match mode {
                0..=2 => assert_eq!(
                    error,
                    CollectionError::Cell(StableCellPayloadError::NotStableCell)
                ),
                3 => assert!(matches!(
                    error,
                    CollectionError::Cell(StableCellPayloadError::InvalidLength { .. })
                )),
                _ => unreachable!(),
            }
            let memory = runtime
                .open_memory(&ic_storage::StableKey::parse("storage.probe.v1").unwrap())
                .unwrap();
            assert_eq!(memory.size(), pages);
            let mut after = vec![0; before.len()];
            memory.read(0, &mut after);
            assert!(before == after, "failed load changed backing bytes");
        });
        reply(protocol::CELL_FAILURE_ID, &());
    }

    #[unsafe(export_name = "canister_post_upgrade")]
    extern "C" fn upgrade() {
        let mode: u8 = argument(protocol::UPGRADE_ID);
        bootstrap();
        if mode != 0 {
            replace(200);
            panic!("injected upgrade trap after collection writes");
        }
    }

    #[unsafe(export_name = "canister_query snapshot")]
    extern "C" fn snapshot() {
        argument::<()>(protocol::SNAPSHOT_ID);
        let values = with_runtime(|runtime| {
            let map = StableBTreeMap::<u64, u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.map.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            let view = region(runtime);
            let cell = StableCell::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.cell.v1").unwrap())
                    .unwrap(),
                16 * 1024 * 1024,
            )
            .unwrap();
            let journal = StableCell::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.journal.v1").unwrap())
                    .unwrap(),
                16 * 1024 * 1024,
            )
            .unwrap();
            assert_eq!(journal.get(), cell.get());
            assert!(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.journal.v1").unwrap())
                    .is_err()
            );
            assert_eq!(
                view.keys_range_limited(10.., 3)
                    .unwrap()
                    .collect::<Vec<_>>(),
                [10, 11, 12]
            );
            let set = StableBTreeSet::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.set.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            let vector = StableVec::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.vector.v1").unwrap())
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                set.iter().collect::<Result<Vec<_>, _>>().unwrap(),
                [map.get(&0).unwrap().unwrap()]
            );
            assert_eq!(
                vector.iter().collect::<Result<Vec<_>, _>>().unwrap(),
                [map.get(&0).unwrap().unwrap()]
            );
            let high = StableCell::<u64>::load(
                runtime
                    .open_memory(&ic_storage::StableKey::parse("storage.extra.n1023.v1").unwrap())
                    .unwrap(),
                16 * 1024 * 1024,
            )
            .unwrap();
            assert_eq!(*high.get(), 1023);
            assert_eq!(map.len(), 128);
            assert_eq!(view.len().unwrap(), 128);
            protocol::Snapshot {
                generation: runtime.committed_generation().unwrap(),
                values: [
                    map.get(&0).unwrap().unwrap(),
                    view.get(&0).unwrap().unwrap(),
                    *cell.get(),
                ],
            }
        });
        reply(protocol::SNAPSHOT_ID, &values);
    }
}
