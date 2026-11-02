//! Identical workload compiled separately for each backend.
#[cfg(target_arch = "wasm32")]
mod canister {
    use ic_platform::ic0;
    use ic_storage_benchmarks::*;
    use serde::{Serialize, de::DeserializeOwned};
    use std::cell::RefCell;

    #[cfg(any(
        all(feature = "owned", feature = "dfinity"),
        not(any(feature = "owned", feature = "dfinity"))
    ))]
    compile_error!("select exactly one backend: owned or dfinity");

    #[cfg(feature = "owned")]
    type Map<K, V> = ic_storage::adapter::StableBTreeMap<K, V>;
    #[cfg(feature = "dfinity")]
    type Map<K, V> = ic_platform::stable_structures::BTreeMap<
        K,
        V,
        ic_platform::stable_structures::memory_manager::VirtualMemory<
            ic_platform::stable_structures::DefaultMemoryImpl,
        >,
    >;
    #[cfg(feature = "dfinity")]
    use ic_platform::stable_structures::Storable;
    #[cfg(feature = "owned")]
    use ic_storage::adapter::Storable;
    thread_local! { static MAP: RefCell<Option<Box<dyn Benchmark>>> = const { RefCell::new(None) }; }

    // A failed Storage operation fails the benchmark; upstream reports failures
    // through traps. Successful-path admission stays inside the timed operation.
    macro_rules! checked {
        ($operation:expr) => {{
            #[cfg(feature = "owned")]
            {
                $operation.expect("benchmark Storage operation")
            }
            #[cfg(feature = "dfinity")]
            {
                $operation
            }
        }};
    }

    struct TypedMap<K: BenchKey + Storable, V: BenchValue + Storable> {
        case: Case,
        map: Map<K, V>,
    }
    trait Benchmark {
        fn setup(&mut self, request: Setup) -> u64;
        fn verify(&self, request: Setup) -> Vec<(u64, Vec<u8>)>;
        fn run(&mut self, request: Request) -> Response;
    }

    fn argument<T: DeserializeOwned>(id: u64) -> T {
        let size = ic0::msg_arg_data_size();
        assert!(size <= LIMITS.bytes);
        let mut bytes = vec![0; size];
        ic0::msg_arg_data_copy(&mut bytes, 0);
        ic_wire::native::decode_request(id, &bytes, LIMITS).unwrap()
    }
    fn reply(id: u64, result: &impl Serialize) {
        let bytes = ic_wire::native::encode_response(id, result, LIMITS).unwrap();
        ic0::msg_reply_data_append(&bytes);
        ic0::msg_reply();
    }
    fn new_map<K: BenchKey + Storable, V: BenchValue + Storable>() -> Map<K, V> {
        #[cfg(feature = "owned")]
        let map = {
            use ic_storage::*;
            let mut runtime = MemoryRuntime::new(DefaultMemoryImpl::default()).unwrap();
            let requests = SealedDeclarationSnapshot::new(vec![
                MemoryRequest::new("benchmark", "benchmark.rows.v1").unwrap(),
            ])
            .unwrap();
            let grants = AllocationGrants::new(vec![
                MemoryAuthority::new("benchmark", "benchmark.").unwrap(),
            ])
            .unwrap();
            runtime
                .bootstrap(&requests, &grants, &GenericAllocationPolicy)
                .unwrap();
            Map::create(
                runtime
                    .open_memory(&StableKey::parse("benchmark.rows.v1").unwrap())
                    .unwrap(),
            )
            .expect("benchmark map creation")
        };
        #[cfg(feature = "dfinity")]
        let map = {
            use ic_platform::stable_structures::{
                DefaultMemoryImpl,
                memory_manager::{MemoryId, MemoryManager},
            };
            let manager = MemoryManager::init(DefaultMemoryImpl::default());
            Map::init(manager.get(MemoryId::new(0)))
        };
        map
    }
    #[unsafe(export_name = "canister_init")]
    extern "C" fn init() {
        let case: Case = argument(0);
        macro_rules! typed {
            ($key:ty, $value:ty) => {
                Box::new(TypedMap::<$key, $value> {
                    case,
                    map: new_map(),
                }) as Box<dyn Benchmark>
            };
        }
        let map = match case {
            Case::U64Bytes => typed!(u64, Vec<u8>),
            Case::U64U64 => typed!(u64, u64),
            Case::U64U128 => typed!(u64, u128),
            Case::Bytes32Bytes32 => typed!([u8; 32], [u8; 32]),
            Case::StringString => typed!(String, String),
            Case::U64Record => typed!(u64, WireRecord),
        };
        MAP.with(|slot| *slot.borrow_mut() = Some(map));
    }
    #[unsafe(export_name = "canister_update setup")]
    extern "C" fn setup() {
        let r: Setup = argument(SETUP_ID);
        assert!(r.count <= 128 && r.start + u64::from(r.count) <= 16_384);
        MAP.with(|slot| reply(SETUP_ID, &slot.borrow_mut().as_mut().unwrap().setup(r)));
    }
    #[unsafe(export_name = "canister_query verify")]
    extern "C" fn verify() {
        let r: Setup = argument(3);
        assert!(r.count <= 128);
        MAP.with(|slot| reply(3, &slot.borrow().as_ref().unwrap().verify(r)));
    }
    #[unsafe(export_name = "canister_update benchmark")]
    extern "C" fn benchmark() {
        let r: Request = argument(RUN_ID);
        MAP.with(|slot| reply(RUN_ID, &slot.borrow_mut().as_mut().unwrap().run(r)));
    }
    impl<K: BenchKey + Storable, V: BenchValue + Storable> Benchmark for TypedMap<K, V> {
        fn setup(&mut self, r: Setup) -> u64 {
            assert!(self.case.payloads().contains(&r.payload));
            for id in r.start..r.start + u64::from(r.count) {
                assert!(
                    checked!(
                        self.map
                            .insert(K::from_id(id), V::make(id, r.payload, false))
                    )
                    .is_none()
                );
            }
            self.map.len()
        }
        fn verify(&self, r: Setup) -> Vec<(u64, Vec<u8>)> {
            self.map
                .range(K::from_id(r.start)..)
                .take(r.count as usize)
                .map(|entry| {
                    let (key, value) = checked!(checked!(entry).into_pair());
                    (key.id(), value.canonical())
                })
                .collect()
        }
        fn run(&mut self, r: Request) -> Response {
            assert_eq!(r.case, self.case);
            assert!(self.case.payloads().contains(&r.payload));
            let ids = keys(r);
            let scan_start = K::from_id(ids[0]);
            // Prepare owned input and result keys before timing: inserts consume keys.
            let inputs: Vec<_> = ids
                .into_iter()
                .map(|id| {
                    (
                        K::from_id(id),
                        K::from_id(id),
                        matches!(r.operation, Operation::Replace | Operation::Insert)
                            .then(|| V::make(id, r.payload, r.operation == Operation::Replace)),
                    )
                })
                .collect();
            let mut entries = Vec::with_capacity(r.count as usize);
            let start = ic0::performance_counter(0);
            if r.operation == Operation::Scan {
                for entry in self.map.range(scan_start..).take(r.count as usize) {
                    let (key, value) = checked!(checked!(entry).into_pair());
                    entries.push((key, Some(value)));
                }
            } else {
                for (key, result_key, value) in inputs {
                    let result = match r.operation {
                        Operation::Control => {
                            std::hint::black_box(&key);
                            None
                        }
                        Operation::Get | Operation::Miss => checked!(self.map.get(&key)),
                        Operation::Replace | Operation::Insert => {
                            checked!(self.map.insert(key, value.unwrap()))
                        }
                        Operation::Remove => checked!(self.map.remove(&key)),
                        Operation::Scan => unreachable!(),
                    };
                    entries.push((result_key, result));
                }
            }
            let instructions = ic0::performance_counter(0) - start;
            Response {
                instructions,
                // Normalization is deliberately after the measured interval.
                entries: entries
                    .into_iter()
                    .map(|(key, value)| (key.id(), value.map(V::canonical)))
                    .collect(),
                length: self.map.len(),
                stable_pages: ic0::stable64_size(),
            }
        }
    }
}
