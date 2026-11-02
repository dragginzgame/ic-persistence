# Installed map comparison

Compare IC Storage with DFINITY's `ic-stable-structures` using the same Rust
workload compiled into separate, original Wasm canisters. Like IC Wire's Candid
comparison, this is an explicitly selected, unpublished benchmark with retained
raw measurements. It is not a product release gate.

From the repository root, with Rust 1.99.1 and its Wasm target already installed:

```sh
# Explicit preparation, only when needed:
make install-runtime-server
cargo fetch --locked --manifest-path testing/runtime-qualification/Cargo.toml

# Offline builds and installed comparison; no optimizer required:
make benchmark-storage
make benchmark-storage BENCH_ROWS=4096 BENCH_SAMPLES=3
```

`BENCH_ROWS` accepts 1–16384 and `BENCH_SAMPLES` accepts 1–20. The defaults are
256 rows and three samples. Sweep population separately: the default is a small
map baseline, not evidence for GiB-scale collections. No tools or dependencies
are installed by the benchmark command. IC Testkit 0.78.0 owns prepared PocketIC
16.1.0 admission and lifecycle; Platform supplies its matching native client.

The independent workspace lockfile selects DFINITY 0.7.2 through IC Platform's
optional `stable-structures` boundary. Production Storage has no such dependency.
Storage release preparation updates its local requirement and lock entry in this
workspace through the existing metadata transaction; external selections stay fixed.
The `comparison` profile uses optimization level 3, fat LTO, one codegen unit and
abort panics for both backends. There is no Binaryen transformation. A backend
feature must be selected explicitly when compiling the Wasm example.

## Workload and validation

Each backend/type/size/sample gets a fresh canister, populated with ascending
keys and matched values in batches of 128. The concrete map type matrix is:

| Key | Value | Logical value bytes |
| --- | --- | ---: |
| `u64` | `Vec<u8>` | 32, 1024 |
| `u64` | `u64` | 8 |
| `u64` | `u128` | 16 |
| `[u8; 32]` | `[u8; 32]` | 32 |
| `String` | `String` | 32, 1024 |
| `u64` | `WireRecord` | 60 |

These are concrete generic map instantiations, not byte-vector substitutes.
Primitive types use each backend's built-in `Storable` implementation. Fixed
array keys have an eight-byte big-endian ID followed by 24 constant bytes;
string keys are `account/` plus a zero-padded 20-digit ID. Both preserve logical
ID order, which the tests check across numeric width boundaries. String values
contain two-byte UTF-8 characters. Scalar values exercise high bits, including
both halves of `u128`.

`WireRecord` contains an account ID, `u128` balance, generation and 32-byte tag.
Both backends use exactly the same bounded IC Wire storage codec for this case.
Its 60 logical field bytes are not its encoded size: the encoding is variable,
bounded at 256 bytes. Replacements keep logical size but may change encoded size.
This case includes application codec cost in the measured map operations.

Each measured call performs one or 32 operations (or returns up to that
many scan rows). Keys use a deterministic generator over the full population or
the first 16 keys. Sample seeds are 1 through the selected sample count. Backend
order alternates by sample.

Calls run in this order: loop control, successful reads, misses, existing-key
replacements, forward bounded scans, fresh inserts, removals. Every
call is a separate update execution. Repeated keys are intentional, especially
in hot batches; repeated removals may miss. Inserts and misses use keys beyond
the initial population. Scans may return fewer rows near the end. These are
matched traces, not seven independently reset states. Maps stay open in heap
between calls; cold reopening and upgrade are outside this workload.

A native `BTreeMap` model checks every returned key/value, displaced value and
length. Final queries compare all remaining rows in bounded chunks. Raw responses
are flushed before comparison; a mismatch or trap fails the run. The summary is
written only after every comparison succeeds.
Concrete keys are converted back to logical IDs and values to canonical bytes
after timing. Record normalization concatenates all fields without invoking its
storage codec, so validation checks decoded contents independently of that codec.
The default matrix has 224 comparison rows and 1,344 measured calls across both
backends, with three seeds per case.

Storage uses a governed allocation admitted through its normal runtime and
exclusive lease. DFINITY uses stock `MemoryManager` defaults and memory ID 0.
Thus this compares the configured storage stacks, including their memory
management, rather than isolating the tree algorithms on an identical allocator.
Storage's owned AVL map and DFINITY's B-tree also have different layouts and
node occupancy. The benchmark does not weaken governance for a lower score.

## Reading the results

Each attempt retains evidence under `target/storage-comparison/attempt.*/`:

- `report/summary.csv`: per-case upper-median instruction counts and the ratio
  `storage_over_dfinity`; below 1 means Storage used fewer measured instructions.
  `case`, `key_type`, `value_type` and `logical_value_bytes` identify each type/size.
- `report/samples.jsonl`: individual requests, instruction counts, complete
  results, lengths and physical stable-memory pages (64 KiB each).
- `report/configuration.json`: selected case and measurement metadata.
- Both Wasm artifacts, compiler identity, locked dependency selections, source
  archive/hashes, pending tracked diff, build logs and managed server logs.

`performance_counter(0)` surrounds the collection operations, including their
stable IO, codecs, allocation, loop and result collection. Request decoding,
typed key/value preparation, result normalization, final validation and response encoding are outside the
counter. Output capacity is reserved before measurement. Returned/displaced
values remain live through the counter endpoint. Preparation can touch heap
regions before measurement; this is not a complete endpoint instruction bill.
All timings are instructions per measured call, not per returned row. A batch
shares an execution's tracking state. Control rows expose loop overhead and are
reported directly, never automatically subtracted.

Type selection dispatch occurs before the timer; traversal uses a concrete map.
The expanded typed harness changes preparation and result handling relative to
the original byte-vector-only harness. Treat saved earlier results as historical
baselines, not a controlled before/after measurement of library changes.

Physical stable pages include directory/manager metadata and reservation; they
are neither live payload size nor a heap footprint. The stock manager reserves
bucket capacity differently from Storage. Setup costs are excluded, but setup's
layout and capacity remain. Query verification runs only after measured work.
There is no timing claim for native host wall-clock speed, Candid versus Wire,
retained heap, large business records, upgrades, or complete application calls.
Keep individual cases and raw samples: an aggregate ratio would hide workload
differences. Installed correctness here supplements, not replaces, the separate
IO, upgrade, rollback and corruption qualification suites.
