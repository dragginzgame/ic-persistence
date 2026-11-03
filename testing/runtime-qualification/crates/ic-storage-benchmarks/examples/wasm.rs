//! Identical workload compiled separately for each backend.
#[cfg(target_arch = "wasm32")]
mod canister {
    use ic_platform_runtime::ic0;
    use ic_storage_benchmarks::*;
    use serde::{Serialize, de::DeserializeOwned};
    use std::cell::RefCell;

    #[cfg(any(
        all(feature = "owned", any(feature = "dfinity", feature = "dfinity-head")),
        all(feature = "dfinity", feature = "dfinity-head"),
        not(any(feature = "owned", feature = "dfinity", feature = "dfinity-head"))
    ))]
    compile_error!("select exactly one backend: owned or dfinity");

    #[cfg(feature = "owned")]
    type Map<K, V> = ic_storage::adapter::StableBTreeMap<K, V>;
    #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
    type Map<K, V> = ic_storage_benchmarks::upstream::BTreeMap<
        K,
        V,
        ic_storage_benchmarks::upstream::memory_manager::VirtualMemory<
            ic_storage_benchmarks::upstream::DefaultMemoryImpl,
        >,
    >;
    #[cfg(feature = "owned")]
    use ic_storage::adapter::Storable;
    #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
    use ic_storage_benchmarks::upstream::Storable;
    thread_local! { static MAP: RefCell<Option<Box<dyn Benchmark>>> = const { RefCell::new(None) }; }

    // A failed Storage operation fails the benchmark; upstream reports failures
    // through traps. Successful-path admission stays inside the timed operation.
    macro_rules! checked {
        ($operation:expr) => {{
            #[cfg(feature = "owned")]
            {
                $operation.expect("benchmark Storage operation")
            }
            #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
            {
                $operation
            }
        }};
    }

    struct TypedMap<K: BenchKey + Storable, V: BenchValue + Storable> {
        case: Case,
        map: Map<K, V>,
        settings: Initialization,
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
        let bytes = ic_wire::native::encode_response(id, result, LIMITS).unwrap_or_else(|error| {
            ic0::trap(format!("benchmark response encoding: {error}").as_bytes())
        });
        ic0::msg_reply_data_append(&bytes);
        ic0::msg_reply();
    }
    #[cfg(feature = "owned")]
    type BenchMemory = ic_storage::RuntimeMemory<ic_storage::DefaultMemoryImpl>;
    #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
    type BenchMemory = ic_storage_benchmarks::upstream::memory_manager::VirtualMemory<
        ic_storage_benchmarks::upstream::DefaultMemoryImpl,
    >;
    fn new_memory(bucket_pages: std::num::NonZeroU16) -> BenchMemory {
        #[cfg(feature = "owned")]
        {
            let _ = bucket_pages;
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
            runtime
                .open_memory(&StableKey::parse("benchmark.rows.v1").unwrap())
                .unwrap()
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        {
            use ic_storage_benchmarks::upstream::{
                DefaultMemoryImpl,
                memory_manager::{MemoryId, MemoryManager},
            };
            MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), bucket_pages.get())
                .get(MemoryId::new(0))
        }
    }
    fn new_map<K: BenchKey + Storable, V: BenchValue + Storable>(
        settings: Initialization,
    ) -> Map<K, V> {
        #[cfg(feature = "owned")]
        {
            Map::create(new_memory(settings.upstream_bucket_pages)).unwrap()
        }
        #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
        {
            Map::init(new_memory(settings.upstream_bucket_pages))
        }
    }
    fn configure_cache<K: BenchKey + Storable, V: BenchValue + Storable>(
        map: &mut Map<K, V>,
        settings: Initialization,
    ) {
        #[cfg(feature = "owned")]
        map.set_node_cache_budget(settings.cache_bytes);
        #[cfg(feature = "dfinity-head")]
        map.node_cache_resize(settings.upstream_cache_slots);
        #[cfg(feature = "dfinity")]
        let _ = (map, settings);
    }
    fn configured_map<K: BenchKey + Storable, V: BenchValue + Storable>(
        settings: Initialization,
    ) -> Map<K, V> {
        let mut map = new_map(settings);
        configure_cache(&mut map, settings);
        map
    }
    mod collections;
    use collections::{Region, TypedCell, TypedSet, TypedVector};
    #[unsafe(export_name = "canister_init")]
    extern "C" fn init() {
        let settings: Initialization = argument(0);
        let case = settings.case;
        macro_rules! typed {
            ($key:ty, $value:ty) => {
                Box::new(TypedMap::<$key, $value> {
                    case,
                    map: configured_map(settings),
                    settings,
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
            Case::SetU64 => Box::new(TypedSet::<u64>::new(case, settings.upstream_bucket_pages)),
            Case::SetString => Box::new(TypedSet::<String>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::VectorU64 => Box::new(TypedVector::<u64>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::VectorBytes32 => Box::new(TypedVector::<[u8; 32]>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::VectorRecord => Box::new(TypedVector::<WireRecord>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::CellBytes => Box::new(TypedCell::<Vec<u8>>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::CellRecord => Box::new(TypedCell::<WireRecord>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::RegionBytes => Box::new(Region::new(settings.upstream_bucket_pages)),
            Case::I64I64 => typed!(Signed64, Signed64),
            Case::U128U128 => typed!(u128, u128),
            Case::Bytes64U128 => typed!([u8; 64], u128),
            Case::U64Fixed31 => typed!(u64, [u8; 31]),
            Case::U64Fixed32 => typed!(u64, [u8; 32]),
            Case::U64Fixed33 => typed!(u64, [u8; 33]),
            Case::U64Fixed256 => typed!(u64, [u8; 256]),
            Case::SetI64 => Box::new(TypedSet::<Signed64>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::SetBytes64 => Box::new(TypedSet::<[u8; 64]>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::VectorI64 => Box::new(TypedVector::<Signed64>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::VectorU128 => Box::new(TypedVector::<u128>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::VectorF64 => Box::new(TypedVector::<f64>::new(
                case,
                settings.upstream_bucket_pages,
            )),
            Case::CellU128 => {
                Box::new(TypedCell::<u128>::new(case, settings.upstream_bucket_pages))
            }
            Case::CellFixed32 => Box::new(TypedCell::<[u8; 32]>::new(
                case,
                settings.upstream_bucket_pages,
            )),
        };
        MAP.with(|slot| *slot.borrow_mut() = Some(map));
    }
    #[unsafe(export_name = "canister_update setup")]
    extern "C" fn setup() {
        let r: Setup = argument(SETUP_ID);
        assert!(r.count <= 128 && r.start + u64::from(r.count) <= u64::from(MAX_ROWS));
        assert!(r.ids.iter().all(|&id| id < u64::from(MAX_ROWS)));
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
            for id in population_ids(&r) {
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
            if self.settings.cold_cache {
                configure_cache(&mut self.map, self.settings);
            }
            assert_eq!(r.case, self.case);
            assert!(self.case.payloads().contains(&r.payload));
            let ids = keys(r);
            let scan_start = K::from_id(ids[0]);
            let scan_end = K::from_id(ids[0] + u64::from(r.count.div_ceil(2)));
            // Prepare owned input and result keys before timing: inserts consume keys.
            let inputs: Vec<_> = ids
                .into_iter()
                .map(|id| {
                    (
                        K::from_id(id),
                        K::from_id(id),
                        matches!(
                            r.operation,
                            Operation::Replace
                                | Operation::Insert
                                | Operation::Reinsert
                                | Operation::Grow
                                | Operation::Shrink
                        )
                        .then(|| V::make(id, payload_for(r), replacement(r.operation))),
                    )
                })
                .collect();
            let mut entries = Vec::with_capacity(r.count as usize);
            let mut booleans = Vec::with_capacity(r.count as usize);
            let mut key_entries = Vec::with_capacity(r.count as usize);
            let start = ic0::performance_counter(0);
            if r.operation == Operation::KeysScan {
                #[cfg(feature = "owned")]
                for key in self.map.keys().take(r.count as usize) {
                    key_entries.push(checked!(key));
                }
                #[cfg(any(feature = "dfinity", feature = "dfinity-head"))]
                for key in self.map.keys().take(r.count as usize) {
                    key_entries.push(key);
                }
            } else if matches!(r.operation, Operation::First | Operation::Last) {
                for _ in 0..r.count {
                    let entry = if r.operation == Operation::First {
                        checked!(self.map.first_key_value())
                    } else {
                        checked!(self.map.last_key_value())
                    };
                    if let Some((key, value)) = entry {
                        entries.push((key, Some(value)));
                    }
                }
            } else if r.operation == Operation::BoundedScan {
                for entry in self.map.range(scan_start..scan_end).take(r.count as usize) {
                    let (key, value) = checked!(checked!(entry).into_pair());
                    entries.push((key, Some(value)));
                }
            } else if r.operation == Operation::ReverseScan {
                for entry in self.map.range(..=scan_start).rev().take(r.count as usize) {
                    let (key, value) = checked!(checked!(entry).into_pair());
                    entries.push((key, Some(value)));
                }
            } else if r.operation == Operation::Scan {
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
                        Operation::Get | Operation::MissAbove => checked!(self.map.get(&key)),
                        Operation::Replace
                        | Operation::Insert
                        | Operation::Reinsert
                        | Operation::Grow
                        | Operation::Shrink => {
                            checked!(self.map.insert(key, value.unwrap()))
                        }
                        Operation::Remove | Operation::RemoveMiss | Operation::RemoveInserted => {
                            checked!(self.map.remove(&key))
                        }
                        Operation::Contains => {
                            let found = checked!(self.map.contains_key(&key));
                            booleans.push((result_key, found));
                            continue;
                        }
                        _ => unreachable!(),
                    };
                    entries.push((result_key, result));
                }
            }
            let instructions = ic0::performance_counter(0) - start;
            Response {
                instructions,
                // Normalization is deliberately after the measured interval.
                entries: if r.operation == Operation::Contains {
                    booleans
                        .into_iter()
                        .map(|(k, v)| (k.id(), Some(vec![u8::from(v)])))
                        .collect()
                } else if r.operation == Operation::KeysScan {
                    key_entries
                        .into_iter()
                        .map(|k| (k.id(), Some(vec![])))
                        .collect()
                } else {
                    entries
                        .into_iter()
                        .map(|(key, value)| (key.id(), value.map(V::canonical)))
                        .collect()
                },
                length: self.map.len(),
                stable_pages: ic0::stable64_size(),
            }
        }
    }
}
