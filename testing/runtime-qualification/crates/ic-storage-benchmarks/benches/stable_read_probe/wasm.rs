//! Read accounting only. The host first populates a governed vector, then upgrades
//! this diagnostic fixture over its unchanged stable image. No collection is opened.
#[cfg(target_arch = "wasm32")]
mod canister {
    use ic_platform_runtime::ic0;
    use ic_storage_benchmarks::read_probe::{ID, LIMITS, Probe, Read};

    fn measure(offset: u64, bytes: &mut [u8]) -> u64 {
        let start = ic0::performance_counter(0);
        ic0::stable64_read(bytes, offset);
        ic0::performance_counter(0) - start
    }

    fn sample(phase: &str, offset: u64, bytes: &[u8], instructions: u64) -> Read {
        Read {
            phase: phase.into(),
            offset,
            destination: bytes.as_ptr() as usize as u64,
            instructions,
            bytes: bytes.into(),
        }
    }

    #[unsafe(export_name = "canister_query probe")]
    extern "C" fn probe() {
        let size = ic0::msg_arg_data_size();
        assert!(size <= 256);
        let mut request = vec![0; size];
        ic0::msg_arg_data_copy(&mut request, 0);
        let probe: Probe = ic_wire::native::decode_request(ID, &request, LIMITS).unwrap();
        assert!((1..=12).contains(&probe.prefix));
        assert!((1..=4096).contains(&probe.payload));
        let payload_offset = probe.offset.checked_add(u64::from(probe.prefix)).unwrap();
        assert!(
            payload_offset
                .checked_add(u64::from(probe.payload))
                .unwrap()
                <= ic0::stable64_size() * 65536
        );
        let mut prefix = vec![0xa5; probe.prefix as usize];
        let mut payload = vec![0xa5; probe.payload as usize];
        let mut combined = vec![0xa5; probe.prefix as usize + probe.payload as usize];
        // Keep initialization before all measured system calls, including heap
        // access/dirty charges. Source stable pages have not been read here.
        std::hint::black_box((&mut prefix, &mut payload, &mut combined));
        let reads = if probe.combined {
            let first = measure(probe.offset, &mut combined);
            let repeat = measure(payload_offset, &mut payload);
            vec![
                sample("combined", probe.offset, &combined, first),
                sample("repeat", payload_offset, &payload, repeat),
            ]
        } else {
            let header = measure(probe.offset, &mut prefix);
            let first = measure(payload_offset, &mut payload);
            let repeat = measure(payload_offset, &mut payload);
            vec![
                sample("prefix", probe.offset, &prefix, header),
                sample("payload", payload_offset, &payload, first),
                sample("repeat", payload_offset, &payload, repeat),
            ]
        };
        let reply = ic_wire::native::encode_response(ID, &reads, LIMITS).unwrap();
        ic0::msg_reply_data_append(&reply);
        ic0::msg_reply();
    }
}
