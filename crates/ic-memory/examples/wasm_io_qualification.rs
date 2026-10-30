//! Installed IO/lifecycle qualification fixture, not an application integration API.
//! Shared bounded Wire messages avoid local packet encoding.

#[cfg(target_arch = "wasm32")]
pub mod support;
#[cfg(target_arch = "wasm32")]
use support::{host, io_protocol as protocol};

#[cfg(target_arch = "wasm32")]
#[expect(
    unsafe_code,
    reason = "test-only raw IC system calls and Memory::read_unsafe"
)]
mod canister {
    use super::host::RUNTIME;
    use super::{pool, protocol};
    use ic_memory::{GenericAllocationPolicy, Memory, MemoryManagerConfig};
    use ic_wire::native;
    use serde::{Serialize, de::DeserializeOwned};

    fn with_runtime<T>(operation: impl FnOnce(&super::host::Runtime) -> T) -> T {
        RUNTIME.with(|slot| operation(slot.borrow().as_ref().expect("initialized fixture runtime")))
    }

    fn open_memory(
        key: &str,
    ) -> Result<ic_memory::RuntimeMemory<ic_memory::DefaultMemoryImpl>, ic_memory::RuntimeOpenError>
    {
        with_runtime(|runtime| runtime.open_memory(key))
    }

    fn memory_id(key: &str) -> Result<u8, ic_memory::RuntimeOpenError> {
        with_runtime(|runtime| runtime.memory_id(key))
    }

    const PAGE: u64 = 65_536;
    const ROWS: &str = "io_qualification.rows.v1";
    const NEIGHBOR: &str = "io_qualification.neighbor.v1";

    fn declarations() -> ic_memory::SealedDeclarationSnapshot {
        ic_memory::SealedDeclarationSnapshot::new(vec![
            ic_memory::MemoryRequest::new("io_qualification", "io_qualification.neighbor.v1")
                .unwrap(),
            ic_memory::MemoryRequest::new("io_qualification", "io_qualification.rows.v1").unwrap(),
        ])
        .unwrap()
    }

    #[link(wasm_import_module = "ic0")]
    unsafe extern "C" {
        fn msg_arg_data_size() -> usize;
        fn msg_arg_data_copy(dst: usize, offset: usize, size: usize);
        fn msg_reply_data_append(src: usize, size: usize);
        fn msg_reply();
        fn performance_counter(counter_type: u32) -> u64;
    }

    fn arguments<T: DeserializeOwned>(id: u64) -> T {
        // SAFETY: Admit the frame length before allocating/copying argument bytes.
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

    fn bootstrap(mode: u8) {
        let pages = if mode == 2 { 16 } else { 8 };
        let mut runtime = super::host::Runtime::new_with_config(
            ic_memory::DefaultMemoryImpl::default(),
            MemoryManagerConfig::new(pages).unwrap(),
        )
        .unwrap();
        runtime
            .bootstrap(&declarations(), &pool(), &GenericAllocationPolicy)
            .unwrap();
        RUNTIME.with(|slot| *slot.borrow_mut() = Some(runtime));
        assert_ne!(mode, 1, "injected trap after durable bootstrap write");
    }

    #[unsafe(export_name = "canister_init")]
    extern "C" fn init() {
        arguments::<()>(protocol::INIT_ID);
        bootstrap(0);
        let neighbor = open_memory(NEIGHBOR).unwrap();
        neighbor.grow(8).unwrap();
        neighbor.write(8 * PAGE - 1, &[0x6D]);
        let rows = open_memory(ROWS).unwrap();
        rows.grow(1).unwrap();
        rows.write(0, &[0x42]);
    }

    #[unsafe(export_name = "canister_post_upgrade")]
    extern "C" fn post_upgrade() {
        bootstrap(arguments(protocol::UPGRADE_ID));
    }

    #[unsafe(export_name = "canister_update composition")]
    extern "C" fn composition() {
        arguments::<()>(protocol::COMPOSITION_ID);
        // Two independent components use one host runtime. Reconstructing the
        // default backing is refused even after that owner drops, until both
        // retained component handles have released custody.
        let runtime = RUNTIME.with(|slot| slot.borrow_mut().take().unwrap());
        let generation = runtime.committed_generation().unwrap();
        let rows = runtime.open_memory(ROWS).unwrap();
        let neighbor = runtime.open_memory(NEIGHBOR).unwrap();
        let physical = ic_memory::DefaultMemoryImpl::default().size();
        let refused = || {
            assert!(matches!(
                super::host::Runtime::new(ic_memory::DefaultMemoryImpl::default()),
                Err(ic_memory::RuntimeConstructionError::BackingAlreadyClaimed)
            ));
        };
        refused();
        rows.write(0, &[0x42]);
        neighbor.write(8 * PAGE - 1, &[0x6D]);
        drop(runtime);
        refused();
        let mut marker = [0];
        rows.read(0, &mut marker);
        assert_eq!(marker, [0x42]);
        drop(rows);
        refused();
        neighbor.read(8 * PAGE - 1, &mut marker);
        assert_eq!(marker, [0x6D]);
        drop(neighbor);
        bootstrap(0);
        assert_eq!(
            with_runtime(|runtime| runtime.committed_generation().unwrap()),
            generation + 1
        );
        assert_eq!(ic_memory::DefaultMemoryImpl::default().size(), physical);
        reply(protocol::COMPOSITION_ID, &());
    }

    #[unsafe(export_name = "canister_update metadata_fault")]
    extern "C" fn metadata_fault() {
        let mode: u8 = arguments(protocol::METADATA_FAULT_ID);
        assert!(mode < 6);
        let rows = open_memory(ROWS).unwrap();
        rows.write(0, &[0xA5]);
        // Infrastructure-only fault injection over the actual canister backing.
        // A successful return would expose a missed checkpoint/poison refusal.
        let backing = ic_memory::DefaultMemoryImpl::default();
        match mode % 3 {
            0 => backing.write(4, &[0, 0]),
            1 => {
                let header_prefix = ic_memory::MemoryAllocationSummary::MANAGER_HEADER_BYTES
                    - u64::from(ic_memory::MemoryAllocationSummary::MEMORIES_MEASURED) * 8;
                backing.write(
                    header_prefix + u64::from(memory_id(ROWS).unwrap()) * 8,
                    &2_u64.to_le_bytes(),
                );
            }
            2 => backing.write(
                ic_memory::MemoryAllocationSummary::METADATA_BYTES_READ - 1,
                &[0],
            ),
            _ => panic!("unknown metadata fault"),
        }
        if mode < 3 {
            assert!(matches!(
                open_memory(NEIGHBOR),
                Err(ic_memory::RuntimeOpenError::Layout(
                    ic_memory::MemoryManagerLayoutError::RuntimeMismatch
                ))
            ));
        } else {
            assert!(matches!(
                rows.grow(0),
                Err(ic_memory::RuntimeGrowError::Layout(
                    ic_memory::MemoryManagerLayoutError::RuntimeMismatch
                ))
            ));
        }
        // The checkpoint has poisoned the owner; even an old handle refuses IO.
        let mut byte = [0];
        rows.read(0, &mut byte);
        reply(protocol::METADATA_FAULT_ID, &byte);
    }

    #[unsafe(export_name = "canister_query snapshot")]
    extern "C" fn snapshot() {
        arguments::<()>(protocol::SNAPSHOT_ID);
        let generation = with_runtime(|runtime| runtime.committed_generation().unwrap());
        // Check assigned identities independently of payload retention.
        assert_eq!(memory_id(NEIGHBOR).unwrap(), 10);
        assert_eq!(memory_id(ROWS).unwrap(), 11);
        let rows = open_memory(ROWS).unwrap();
        let neighbor = open_memory(NEIGHBOR).unwrap();
        assert!(matches!(
            open_memory(ROWS),
            Err(ic_memory::RuntimeOpenError::AlreadyOpen { id: 11 })
        ));
        let mut markers = [0; 2];
        rows.read(0, &mut markers[..1]);
        neighbor.read(8 * PAGE - 1, &mut markers[1..]);
        let summary = with_runtime(|runtime| runtime.memory_allocation_summary().unwrap());
        reply(
            protocol::SNAPSHOT_ID,
            &protocol::Snapshot {
                generation,
                markers,
                physical_pages: summary.physical_extent.wasm_pages,
                rows_pages: rows.size(),
            },
        );
    }

    #[unsafe(export_name = "canister_update io")]
    extern "C" fn io() {
        let protocol::IoRequest {
            operation,
            count,
            offset,
        } = arguments(protocol::IO_ID);
        let count = usize::from(count);
        assert!(count <= 2);
        let rows = open_memory(ROWS).unwrap();
        // Warm the cache and dirty valid state before the candidate failure.
        // A trap must roll this mutation back as well as reject the invalid IO.
        rows.write(0, &[0x99]);
        let mut dst = [0; 2];
        // SAFETY: Counter zero is supported in update execution.
        let start = unsafe { performance_counter(0) };
        for _ in 0..1000 {
            match operation {
                0 => rows.read(offset, &mut dst[..count]),
                // SAFETY: count fits the separate writable destination.
                1 => unsafe { rows.read_unsafe(offset, dst.as_mut_ptr(), count) },
                2 => rows.write(offset, &[0xA5; 2][..count]),
                _ => panic!("unknown IO operation"),
            }
        }
        // SAFETY: Same counter/execution as above.
        let instructions = unsafe { performance_counter(0) } - start;
        reply(
            protocol::IO_ID,
            &protocol::IoReply {
                bytes: dst,
                instructions,
            },
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("io_qualification", "io_qualification.").unwrap()],
        vec![],
    )
    .unwrap()
}
