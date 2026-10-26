//! Installed collection/lease qualification; no application codec or recovery policy.

#[cfg(target_arch = "wasm32")]
#[expect(
    unsafe_code,
    reason = "fixture-only raw IC entrypoints and reply calls"
)]
mod canister {
    use ic_memory::{
        DefaultMemoryImpl, MemoryManagerConfig, MemoryRuntime, testing::fixture_runtime,
    };
    use ic_storage::adapter::{BTreeMapRegion, BoundedCell, StableBTreeMap};
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

    fn argument() -> u8 {
        let mut byte = 0;
        // SAFETY: Validate the argument length before copying into one byte.
        unsafe {
            assert_eq!(msg_arg_data_size(), 1);
            msg_arg_data_copy((&raw mut byte) as usize, 0, 1);
        }
        byte
    }

    fn reply(bytes: &[u8]) {
        // SAFETY: The live slice remains readable until the synchronous copy ends.
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
            &["storage.map.v1", "storage.cell.v1", "storage.region.v1"],
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
            assert!(runtime.open_memory("storage.map.v1").is_err());
        });
    }

    #[unsafe(export_name = "canister_update mutate")]
    extern "C" fn mutate() {
        let mode = argument();
        replace(100);
        assert_eq!(mode, 0, "injected trap after collection writes");
        reply(&[]);
    }

    #[unsafe(export_name = "canister_post_upgrade")]
    extern "C" fn upgrade() {
        bootstrap();
        if argument() != 0 {
            replace(200);
            panic!("injected upgrade trap after collection writes");
        }
    }

    #[unsafe(export_name = "canister_query snapshot")]
    extern "C" fn snapshot() {
        let values = with_runtime(|runtime| {
            let map =
                StableBTreeMap::<u64, u64>::load(runtime.open_memory("storage.map.v1").unwrap());
            let view = region(runtime);
            let cell = BoundedCell::init(runtime.open_memory("storage.cell.v1").unwrap(), 0_u64);
            assert_eq!(
                view.keys_range_limited(10.., 3).collect::<Vec<_>>(),
                [10, 11, 12]
            );
            assert_eq!(map.len(), 128);
            assert_eq!(view.len(), 128);
            [
                runtime.committed_generation().unwrap(),
                map.get(&0).unwrap(),
                view.get(&0).unwrap(),
                *cell.get(),
            ]
        });
        reply(
            &values
                .into_iter()
                .flat_map(u64::to_le_bytes)
                .collect::<Vec<_>>(),
        );
    }
}
