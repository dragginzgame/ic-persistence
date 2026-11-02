# Installed collection comparison

Compare IC Storage and DFINITY `ic-stable-structures` 0.7.2 using matched concrete
Rust workloads compiled into separate original Wasm canisters. This explicitly
selected, unpublished benchmark retains raw results and fails on mismatches.
It is not a release gate or a claim about complete application calls.

```sh
# Explicit preparation only when needed:
make install-runtime-server
cargo fetch --locked --manifest-path testing/runtime-qualification/Cargo.toml

make benchmark-storage                             # 256 rows, three seeds
make benchmark-storage BENCH_ROWS=4096 BENCH_SAMPLES=3
make benchmark-storage BENCH_ROWS=64 BENCH_SAMPLES=1 # quick complete smoke matrix
```

Rows accept 1–16384, samples 1–20. Cell always has one value, regardless of the
selected population. Builds are locked/offline on Rust 1.99.1; the command never
installs tools or changes locks. Testkit 0.79.0 manages the prepared PocketIC
16.1.0 server through Platform 0.18's owned native client. Platform 0.18 removed
its stable-structures facade, so only the DFINITY backend retains the named
`ic-platform-stable-benchmark` alias for Platform 0.17. Both backends use Runtime
0.18 for system calls. The production graph has no DFINITY dependency. Both Wasm backends use the same comparison profile:
optimization level 3, fat LTO, one codegen unit, abort panics, no Binaryen pass.

## Supported structures and concrete types

| Structure | Types | Logical value bytes |
| --- | --- | --- |
| Map | u64 → bytes | 0, 31, 32, 33, 1024, 4096 |
| Map | String → String | 0, 32, 1024, 4096 |
| Map | u64 → u64/u128; u128 → u128 | 8; 16 |
| Map | Signed64 → Signed64 | 8 |
| Map | [u8;32] → [u8;32]; [u8;64] → u128 | 32; 16 |
| Map | u64 → [u8;31/32/33/256]; u64 → WireRecord | 31, 32, 33, 256; 60 |
| Set | u64; String; Signed64; [u8;64] | unit values |
| Vector | u64; Signed64; u128; f64; [u8;32]; WireRecord | 8; 8; 16; 8; 32; 60 |
| Cell | bytes; WireRecord; u128; [u8;32] | 0, 31, 32, 33, 1024, 4096; 60; 16; 32 |
| Map region | u64 → bytes | 0, 31, 32, 33, 1024, 4096 |

All are real generic instantiations. `WireRecord` has an account ID, u128 balance,
generation and 32-byte tag, using the same IC Wire codec on both backends. Its
60 field bytes are not its encoded size; the declared encoding ceiling is 256
bytes. Fixed array keys and zero-padded string keys preserve logical ID order.
String values use two-byte UTF-8 characters. Scalars exercise high bits.
There are 28 concrete cases and 46 type/size combinations. The 31/32/33-byte
fixed and variable Map values straddle Storage's inline-value threshold; fixed
256-byte values and 4 KiB variable values extend the payload comparison.

`Signed64` wraps i64 because upstream has no primitive i64 Storable implementation.
Both backends use the same eight-byte little-endian benchmark codec. Signed keys
map logical ID 32 to zero, so populations above 32 cross the sign boundary. The
u128 keys vary their upper 64 bits, and 64-byte keys share a 56-byte prefix before
the ordered ID. Float vectors preserve exact IEEE bits, including signed zeros,
subnormals, infinities and a NaN payload; floats are never ordering keys.

Each case uses uniform or hot-16 keys, one or 32 operations per execution, and
seeds 1 through the requested sample count. Backend order alternates by sample.
Initial population is ascending, in batches of 128. Each complete case/sample/
backend gets fresh state; operations within it share the preceding operation's
committed state. This is a trace, not an independent reset per operation.

- Map: control, hit, miss, replacement, forward scan, insertion, removal,
  membership, reverse scan, key-only scan, growth, shrink, missing removal,
  reinsertion, first/last entry lookups and bounded forward ranges.
  Growth/shrink apply only to variable byte/string values.
- Set: control, membership hit/miss, forward iteration, insertion, removal,
  missing removal and reinsertion. DFINITY's Set iterator is not double-ended;
  reverse Set iteration remains native correctness coverage, not a fake comparison.
- Vector: control, indexed hit/miss, indexed set, forward/reverse iteration,
  push and pop. Pushes extend the current length; pops remove those appended slots.
- Cell: control, cached get and replacement; byte cells also grow/shrink. Gets
  clone the cached value on both sides to retain a result, so they include cloning.
- Region: control, hit/miss, replacement, bounded key snapshot, insertion,
  removal, missing removal, reinsertion, growth and shrink.

Growth replaces values with four times the initial size (minimum 128 bytes);
shrink uses empty bytes/strings. Fresh inserts and missing removals address keys
beyond the initial population. Reinsert revisits the earlier removed original
keys. Repeated keys, missing removals and partial scans are intentional.
Map forward/reverse scans seek from the generated key; Map key-only scans and
Set/Vector iteration start at their respective ends. Region key snapshots seek
from the generated key and enforce the limit within the operation. Bounded Map
scans use `[start, start + ceil(count/2))` and also enforce the requested count;
this exercises an exclusive upper bound independently of the output limit.
First/last lookups run `count` times and retain every result. These three new
operations run after the existing mutation trace, so they observe its final values.

The full matrix has **2,080 comparison rows and 12,480 measured calls** with three
seeds. The one-seed smoke matrix has 4,160 calls. These totals include controls.

## Comparison boundaries

Storage uses admitted governed leases. DFINITY uses its stock MemoryManager
and memory ID 0. This measures their configured stacks, not tree algorithms over
identical allocators. The new Storage B-tree keeps values up to 32 bytes inline
and checksums external values separately. DFINITY has its own node/page layout.

DFINITY has no `BTreeMapRegion`: that control reloads a BTreeMap from the same
VirtualMemory for each operation. Storage uses its actual region API, which also
loads the retained root per operation. Region key results are bounded owned
snapshots on both sides. This is a documented wrapper comparison, not a claim
that upstream supplies the same governance or region bounds.

`performance_counter(0)` surrounds collection operations, their IO, codecs,
allocation, loops and result collection. Decoding the request, generating typed
inputs, reserving output capacity, normalizing results, verification and encoding
the reply are outside timing. Returned values stay live through the endpoint.
Preparation can warm heap regions before the counter. Each measured call is a
separate IC update; batches share that execution's tracking state. Controls are
reported without automatic subtraction.

The native ordered model checks every result and length, including displaced
values, booleans, indexed writes and append/pop behavior. Final bounded queries
compare all remaining contents. Raw samples are flushed before validation; a
mismatch or trap fails the run, and no successful summary is written. Typed
normalization happens after timing; Wire records normalize fields without their
storage codec. Physical stable pages include metadata and reservation, not just
payload bytes. The benchmark protocol admits four million traversal nodes and
128 MiB of conservative allocation allowance so 512 KiB full-result responses fit;
this allowance is not an actual allocation. Request/reply encoding is outside timing.
Native regression tests round-trip the largest verification and grown-value
responses under the same limits. Setup cost is excluded, but its allocation history remains.

This matrix does not measure cold reopening for ordinary collections, setup,
upgrades, rollback, randomized population histories, GiB maps, retained heap or
complete application endpoints. Native B-tree tests additionally cover shuffled
insert/delete histories; installed lifecycle qualification is a separate suite.

## Retained results

Each `target/storage-comparison/attempt.*/` contains:

- `report/summary.csv`: per-case upper-median instructions and
  `storage_over_dfinity` (below 1 means fewer Storage instructions), with explicit
  structure/type/payload/operation/batch/access-pattern labels. The final two
  columns report each backend's median physical stable pages after the operation.
  Each page is 64 KiB; these are whole-canister high-water sizes, including manager
  metadata and reserved capacity, not live payload bytes or allocation deltas.
- `report/samples.jsonl`: requests, counts, full results, lengths and stable pages.
- `report/configuration.json`: case operations, actual population and measurement
  scope; both Wasm files, source archive/hashes, compiler identities, both locked
  graphs, pending diff and build/server logs accompany the report.

New cases, endpoint/range operations and result handling change the harness from
the earlier map-only and 14-case experiments. Older results are historical evidence, not a controlled before/after
speedup for the library. Compare the two backends within a matched current run;
retain per-case regressions rather than averaging them into one score.
