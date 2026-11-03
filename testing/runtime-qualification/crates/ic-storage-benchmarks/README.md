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

Rows accept 1–1,000,000, samples 1–20. Cell always has one value, regardless of the
selected population. Builds are locked/offline on Rust 1.99.1; the command never
installs tools or changes locks. Testkit 0.79.0 manages the prepared PocketIC
16.1.0 server through Platform 0.19's owned native client. Platform 0.18 removed
its stable-structures facade, so only the DFINITY backend retains the named
`ic-platform-stable-benchmark` alias for Platform 0.17. Both backends use Runtime
0.19 for system calls. Client setup and installation failures stop the comparison;
they are not retried or treated as collection results. The production graph has
no DFINITY dependency. Both Wasm backends use the same comparison profile:
optimization level 3, fat LTO, one codegen unit, abort panics, no Binaryen pass.

### Extended controls

The independent workspace also pins upstream Git revision
`b5d1650463f477affb3d358e86b1113953fc9937` as an explicitly optional research
control. `BENCH_REFERENCE=dfinity-head` selects it; the default `dfinity` control
remains published 0.7.2. The Git control does not enter production qualification
graphs. Prepare its locked source with the existing explicit fetch command.

```sh
BENCH_REFERENCE=dfinity-head BENCH_CASES=U64U64,VectorU64 \
BENCH_PATTERNS=uniform,hot16,zipf BENCH_SHUFFLED=1 BENCH_ROUNDS=3 \
BENCH_CACHE_BYTES=131072 BENCH_UPSTREAM_CACHE_SLOTS=32 \
make benchmark-storage BENCH_ROWS=4096 BENCH_SAMPLES=1
```

Case selection uses the exact case names from `types.rs`. Cache settings apply
to ordinary maps only: Storage accepts a byte budget and current upstream accepts
a slot count. These are different limits, not equivalent heap budgets. Both
default to zero in the comparison; published 0.7.2 has no node-cache setting.
Other structures retain their implementation defaults. `BENCH_COLD_CACHE=1`
clears ordinary-map caches before each timed call; clearing cost is excluded.
This measures cold routing, not collection reopening or whole-call cost.

`BENCH_UPSTREAM_BUCKET_PAGES=1..65535` selects the ICSS MemoryManager bucket size
for every structure; the default remains 128 (8 MiB). One Wasm page is 64 KiB.
This changes only the DFINITY backend's backing allocation policy, not its node
size, Storage's growth policy or cache settings. Each fresh comparison canister
receives the selection during initialization. Existing managers cannot be resized
by reopening with a different setting. Smaller buckets reduce reservation slack,
but consume the fixed 32,768-bucket manager limit sooner and can change IO costs.

```sh
for pages in 1 8 128; do
    BENCH_UPSTREAM_BUCKET_PAGES=$pages BENCH_CASES=U64U64 \
    BENCH_PATTERNS=uniform,hot16,zipf BENCH_SHUFFLED=1 BENCH_ROUNDS=3 \
    make benchmark-storage BENCH_ROWS=4096 BENCH_SAMPLES=1
done
```

Compare runs made with the same sources, Wasm files and workload settings.
`configuration.json` and the appended `upstream_bucket_pages` CSV column record
the choice. Setup/growth during population remains outside instruction timing;
reported stable pages include that reservation. This unpublished initialization
protocol changed, so rebuild both Wasm backends and the driver together.

Shuffling uses a deterministic Fisher–Yates permutation for Map, Set and Region
population; Vector/Cell setup retains its semantics. Zipf uses a discrete
harmonic CDF with exponent one, generated outside timing on both backends.
Trace generation can warm heap regions. `BENCH_ROUNDS=1..20` repeats the complete
operation sequence on existing state with different access seeds. Raw samples
retain the round; summaries take the upper median across samples and rounds.
Configuration records every selection. Larger supported populations are opt-in
and must be qualified separately; use selected cases to bound runtime and memory.

## Consumer workloads

The [local memory inventory](../../../../docs/local-memory-audit.md#complete-declaration-inventory-and-repeatable-workloads)
selects three fixed recipes. Use a name instead of repeating a collection of
flags; these are benchmark selections, not new Storage runtime parameters.

| `BENCH_WORKLOAD` | Population | Values / structures | Access |
| --- | --- | --- | --- |
| `control` | 16 records; Cell has one | 256 B / 4 KiB, Map and Cell | Single, uniform, three rounds |
| `records` | 4,096 records | Bytes at 31/32/33/256/1,024/4,096 B; fixed 32-byte Map, 64-byte Set, WireRecord Vector | Batches 1/32, uniform/hot16/Zipf, shuffled Map/Set, three rounds |
| `bulk` | Eight records; Cell has one | 64 KiB / 1 MiB, Map/Vector/Cell/Region | Single, uniform, one round |

```sh
make benchmark-storage BENCH_WORKLOAD=records BENCH_SAMPLES=1 BENCH_UPSTREAM_BUCKET_PAGES=8
# Same recipe and binaries, varying only the ICSS bucket control:
for pages in 1 8 128; do
    make benchmark-storage BENCH_WORKLOAD=bulk BENCH_SAMPLES=1 BENCH_UPSTREAM_BUCKET_PAGES=$pages
done
```

A recipe owns population (so `BENCH_ROWS` is unused), cases, payload sizes,
batches, patterns, ordering, rounds and disabled caches. Combining it with the
other detailed workload environment knobs refuses. Samples and ICSS bucket size
remain explicit experimental controls. Configuration records the recipe and all
resolved settings; original manual selections remain available without a recipe.

Large byte values are opt-in; the ordinary matrix retains its previous payload
sizes. Setup and final verification target at most 512 KiB per call, except for
one indivisible 1 MiB value. Protocol tests qualify that value under bounded Wire
limits. The measured bulk call contains one operation; `Grow` at/above 64 KiB
replaces at the same size, while `Shrink` still writes an empty payload. This
preserves the old trace's growth ceiling without shrinking a 1 MiB value in the
phase named `Grow`. Encoding and setup remain outside the measured interval.

These are synthetic collection traces informed by actual declarations. They do
not reproduce a complete canister, concurrently open allocations, actual live
populations or the 2–32 MiB upper bounds of some control snapshots. The inventory
explains which consumers and bounds are covered; no production default follows
from a successful smoke run.

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
There are 28 concrete cases and 46 type/size combinations in the ordinary matrix.
The additional `VectorBytes` case is opt-in: both backends declare a variable-length
1 MiB maximum (`ChunkBytes`), matching Canic’s chunk vector. ICSS slots reserve
that maximum even for shorter values; this is part of the comparison, not an
equal-layout claim. The 31/32/33-byte
fixed and variable Map values straddle Storage's inline-value threshold; fixed
256-byte values and 4 KiB variable values extend the payload comparison.

`Signed64` wraps i64 because upstream has no primitive i64 Storable implementation.
Both backends use the same eight-byte little-endian benchmark codec. Signed keys
map logical ID 32 to zero, so populations above 32 cross the sign boundary. The
u128 keys vary their upper 64 bits, and 64-byte keys share a 56-byte prefix before
the ordered ID. Float vectors preserve exact IEEE bits, including signed zeros,
subnormals, infinities and a NaN payload; floats are never ordering keys.

By default each case uses uniform or hot-16 hit keys, one or 32 operations per execution, and
seeds 1 through the requested sample count. Backend order alternates by sample.
Default population is ascending, in batches of 128. Each complete case/sample/
backend gets fresh state; operations within it share the preceding operation's
committed state. This is a trace, not an independent reset per operation.

- Map: control, hit, miss, replacement, forward scan, insertion,
  membership, reverse scan, key-only scan, growth, shrink, removal, missing removal,
  reinsertion, inserted-key cleanup, first/last entry lookups and bounded forward ranges.
  Growth/shrink apply only to variable byte/string values.
- Set: control, membership hit/miss, forward iteration, insertion, removal,
  missing removal, reinsertion and inserted-key cleanup. DFINITY's Set iterator is not double-ended;
  reverse Set iteration remains native correctness coverage, not a fake comparison.
- Vector: control, indexed hit/miss, indexed set, forward/reverse iteration,
  push and pop. Pushes extend the current length; pops remove those appended slots.
- Cell: control, cached get and replacement; byte cells also grow/shrink. Gets
  clone the cached value on both sides to retain a result, so they include cloning.
- Region: control, hit/miss, replacement, bounded key snapshot, insertion,
  growth/shrink, removal, missing removal, reinsertion and inserted-key cleanup.

Growth replaces values with four times the initial size (minimum 128 bytes);
shrink uses empty bytes/strings. Growth/shrink run while selected original keys
still exist; removal follows, then reinsertion restores them. `RemoveInserted`
cleans up the insertion range so later rounds start with the original population.
`RemoveMiss` uses a separate never-inserted range. `MissAbove` probes beyond the
population; these misses and insertion/cleanup keys are sequential regardless of
the hit-pattern label. Below-population and interior misses are not represented.
Phase-entry assertions admit existence/absence and previous grow/shrink values
before either backend executes. Duplicate hot keys remain intentional: later
removals/reinsertions within a batch can be misses/replacements. This is still a
stateful mixed trace, not an independent reset per operation.
Map forward/reverse scans seek from the generated key; Map key-only scans and
Set/Vector iteration start at their respective ends. Region key snapshots seek
from the generated key and enforce the limit within the operation. Bounded Map
scans use `[start, start + ceil(count/2))` and also enforce the requested count;
this exercises an exclusive upper bound independently of the output limit.
First/last lookups run `count` times and retain every result. These three new
operations run after the existing mutation trace, so they observe its final values.

Call counts follow the operations and payload sizes recorded in each run's
configuration, multiplied by batches, patterns, samples, rounds and backends.
Controls are included. The extra cleanup phase changes counts from older runs.

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
payload bytes. The benchmark protocol admits eight million traversal nodes and
256 MiB of conservative allocation allowance so bounded responses including a single 1 MiB bulk value fit;
this allowance is not an actual allocation. Request/reply encoding is outside timing.
Native regression tests round-trip the largest verification and grown-value
responses under the same limits. Setup cost is excluded, but its allocation history remains.

This matrix does not measure cold reopening for ordinary collections, setup,
upgrades, rollback, GiB maps, retained heap or
complete application endpoints. Native B-tree tests additionally cover shuffled
insert/delete histories; installed lifecycle qualification is a separate suite.

## Retained results

Each `target/storage-comparison/attempt.*/` contains:

- `report/summary.csv`: per-case upper-median instructions and
  `storage_over_dfinity` (below 1 means fewer Storage instructions), with explicit
  structure/type/payload/operation/batch/access-pattern labels. The two
  page columns report each backend's median physical stable pages after the operation.
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

The phase-correctness repair also changes work ordering and cache history. Earlier
`RemoveMiss` rows actually removed the just-inserted keys; variable-value `Grow`
reinserted removed keys, and `Reinsert` replaced values restored by growth. Their
timings remain valid for those historical mixed traces, not for the old labels.
Use corrected baseline/candidate runs to attribute new implementation gains.
