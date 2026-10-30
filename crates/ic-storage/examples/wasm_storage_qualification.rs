//! Installed collection/lease qualification; no application codec or recovery policy.

#[cfg(target_arch = "wasm32")]
mod support;
#[cfg(target_arch = "wasm32")]
use support::qualification_protocol as protocol;

#[cfg(target_arch = "wasm32")]
#[expect(
    unsafe_code,
    reason = "fixture-only raw IC entrypoints and reply calls"
)]
mod canister {
    use super::protocol;
    use ic_memory::{
        DefaultMemoryImpl, MemoryManagerConfig, MemoryRuntime, StableCellPayloadError,
        testing::fixture_runtime,
    };
    use ic_storage::adapter::{BTreeMapRegion, BoundedCell, Memory, StableBTreeMap, StableCell};
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
        let runtime = fixture_runtime(
            DefaultMemoryImpl::default(),
            "storage",
            "storage.",
            &[
                "storage.map.v1",
                "storage.cell.v1",
                "storage.region.v1",
                "storage.journal.v1",
                "storage.probe.v1",
            ],
            MemoryManagerConfig::new(8).unwrap(),
        )
        .unwrap();
        RUNTIME.with(|slot| *slot.borrow_mut() = Some(runtime));
    }

    fn region(runtime: &MemoryRuntime<DefaultMemoryImpl>) -> BTreeMapRegion<u64, u64> {
        BTreeMapRegion::new(
            Rc::new(runtime.open_memory("storage.region.v1").unwrap()),
            0..64,
        )
    }

    #[unsafe(export_name = "canister_init")]
    extern "C" fn init() {
        argument::<()>(protocol::INIT_ID);
        bootstrap();
        with_runtime(|runtime| {
            let mut map =
                StableBTreeMap::<u64, u64>::new(runtime.open_memory("storage.map.v1").unwrap());
            let mut view = region(runtime);
            view.initialize();
            for key in 0..128 {
                map.insert(key, key + 10);
                view.insert(key, key + 20);
            }
            let _ = BoundedCell::init(runtime.open_memory("storage.cell.v1").unwrap(), 30_u64);
            let _ = StableCell::new(runtime.open_memory("storage.journal.v1").unwrap(), 30_u64);
        });
    }

    fn replace(value: u64) {
        with_runtime(|runtime| {
            let mut map =
                StableBTreeMap::<u64, u64>::load(runtime.open_memory("storage.map.v1").unwrap());
            map.insert(0, value);
            region(runtime).insert(0, value + 1);
            BoundedCell::init(runtime.open_memory("storage.cell.v1").unwrap(), 0_u64)
                .set(value + 2);
            StableCell::<u64>::load(runtime.open_memory("storage.journal.v1").unwrap())
                .unwrap()
                .set(value + 2);
            assert!(runtime.open_memory("storage.map.v1").is_err());
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
            let memory = runtime.open_memory("storage.probe.v1").unwrap();
            if mode != 0 {
                if memory.size() == 0 {
                    memory.grow(1).unwrap();
                }
                let header = match mode {
                    1 => *b"BTR\x01\0\0\0\0",
                    2 => *b"SCL\x02\0\0\0\0",
                    3 => *b"SCL\x01\xff\xff\xff\xff",
                    4 => *b"SCL\x01\x01\0\0\0", // u64 codec panic; host rolls back.
                    _ => panic!("unknown probe mode"),
                };
                memory.write(0, &header);
            }
            let pages = memory.size();
            let mut before = vec![0; usize::try_from(pages * 65_536).unwrap()];
            memory.read(0, &mut before);
            let error = StableCell::<u64>::load(memory).err().unwrap();
            match mode {
                0 | 1 => assert_eq!(error, StableCellPayloadError::NotStableCell),
                2 => assert_eq!(
                    error,
                    StableCellPayloadError::UnexpectedLayoutVersion { version: 2 }
                ),
                3 => assert!(matches!(
                    error,
                    StableCellPayloadError::InvalidLength { .. }
                )),
                _ => unreachable!(),
            }
            let memory = runtime.open_memory("storage.probe.v1").unwrap();
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
            let map =
                StableBTreeMap::<u64, u64>::load(runtime.open_memory("storage.map.v1").unwrap());
            let view = region(runtime);
            let cell = BoundedCell::init(runtime.open_memory("storage.cell.v1").unwrap(), 0_u64);
            let journal =
                StableCell::<u64>::load(runtime.open_memory("storage.journal.v1").unwrap())
                    .unwrap();
            assert_eq!(journal.get(), cell.get());
            assert!(runtime.open_memory("storage.journal.v1").is_err());
            assert_eq!(
                view.keys_range_limited(10.., 3).collect::<Vec<_>>(),
                [10, 11, 12]
            );
            assert_eq!(map.len(), 128);
            assert_eq!(view.len(), 128);
            protocol::Snapshot {
                generation: runtime.committed_generation().unwrap(),
                values: [map.get(&0).unwrap(), view.get(&0).unwrap(), *cell.get()],
            }
        });
        reply(protocol::SNAPSHOT_ID, &values);
    }
}
