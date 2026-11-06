# Storage backing parity with ICSS

Reference: published **ic-stable-structures 0.7.2**, not earlier Storage. Unreleased
changes preserve layouts, APIs and governance. Small fixed Vector reads now save
6,400 instructions per 32 reads, including the full synthetic handler. The older
eight-page gap is a timer-boundary effect, not a handler speedup. Neither measures Auth.

The [step/round matrices](../../measurements/storage-icss-backing-steps.csv) and
[payload matrix](../../measurements/storage-icss-backing-payloads.csv) retain both costs.
Raw attempts: `target/evidence/storage-comparison/`; ICSS buckets: eight pages unless specified, default 128.

## Steps 1–2: isolate backing and remove repeated admission

`dfinity-governed` places published ICSS on Storage memory; differing Rc/codec
wrappers prevent an exact algorithm-only control. Regions reopen per operation;
ICSS has no bounded region API. Retaining admitted directory indices, inlining
cache hits and checking static region bounds before full lease admission reduce
repeated work. Tests cover bounds/custody/poisoning, 512 later bindings,
interleaved growth and detached ownership. The CSV retains full matrices from
`WQ6DQa`, `J3enEn`, `ChfB4h`; step 2's individual effects are not isolated.

## Step 3: restore ICSS's direct reads

Backing had inherited the copied Memory trait's zero-before-read fallback.
Forwarding `read_unsafe` through governed leases and regions restores ICSS's
uninitialized-buffer path. Existing custom public backings retain a default;
private engine views must implement the hook explicitly. Successful reads fully
initialize the destination; failed native partial IO still poisons leases.

Matched depth results below use **shuffled** insertion, 4,096 records, uniform
batches of 32 and three samples/rounds. Cell retains one value. Sources are
`j0xUKb`, `uZ0Imy`, `Zny95C`; each depth run checks 11,808 calls. Both populations
and additional types are in the selected step matrix.

| Operation | Before backing work | Index/bounds | Direct read | ICSS |
| --- | ---: | ---: | ---: | ---: |
| u64 Map Get | 1,235,681 | 1,231,303 | 995,006 | 930,341 |
| u64 Map Replace | 2,122,543 | 2,039,588 | 1,769,297 | 1,650,756 |
| Set Get | 1,136,425 | 1,132,173 | 892,936 | 838,132 |
| Vector Get | 101,880 | 101,752 | 101,432 | 102,933 |
| Cell Get | 3,639 | 3,643 | 3,643 | 4,349 |
| Region Get | 1,766,548 | 1,365,035 | 1,134,372 | 954,675 |
| Region Replace | 2,890,110 | 2,223,279 | 1,960,260 | 1,674,410 |

Ordinary-insertion control `yh1wyH` checks 1,152 calls. Storage versus ICSS/governed:
Map Get 565,640/579,637; Replace 1,164,971/1,208,480; region Get 648,418/601,845;
Replace 1,314,632/1,218,799. Region overhead remains even on the same backing.
Payload shards `l0wLHL`/`Suq31G` check 7,776 calls across 31-byte/4-KiB values.

## Step 4: reject further index/bounds tuning

`3P6tEd` passes 11,808 comparisons but passing indices through all IO and storing
region byte bounds regresses Map/Set while improving region Get under 0.5%.
This batch was reverted byte-for-byte; the CSV retains both sides.

## Step 5: locate the wider Vector gap

The `vector-reads` preset fixes 256 rows, scalar/fixed/chunk/Wire values,
uniform/hot-16 access, batches 1/32 and three mutation rounds. Separate reads
after Set/Push/Pop locate within-round changes; control `lfj2rs` passes 1,440 calls.

After push Storage stays flat; ICSS's eight-page inner loop becomes 160k cheaper.
Both modules are byte-identical across mixed, four-operation and checkpoint runs.
Checkpoint runs `VtOuHy`/`mUd1uV`/`2wi3e4` each pass 2,520 calls. The inner-loop
5.6% uniform/11.0% hot-16 gaps belong to the eight-page control; default-bucket
costs are within 0.13%. Full-handler attribution below changes their interpretation.

## Steps 6–7: trace allocation, then correct the measurement boundary

The frozen-Wasm probes append globals/functions without moving existing memory,
data or globals. Binaryen 133 uses no optimization passes; the plain WAT round
trip reproduces all 840 sample-0 results and instruction counts exactly.
The first probe finds unchanged stable-read cost but 159,968 fewer malloc
instructions and 69 more free instructions. Three bucket controls pass 2,520 calls.

A second probe identifies the costly allocator stores in Rust 1.99.1's dlmalloc
0.2.14. Before growth, ICSS splits the top chunk and pays two approximately 160k
header writes inside the timer. After growth, it uses the designated free chunk:
a footer write at heap offset 8,659,336 touches region 132 **before** the timer,
leaving only one costly header write inside it. The charge moves between stages.
Storage retains the same two costly timed header writes. The two allocator-access
runs each verify 840 calls. Per-access counts include probe overhead.

A separate boundary-only probe measures the complete benchmark handler, including
request decoding, input preparation, response encoding/reply and cleanup. It
preserves **all original inner-loop counts** across 2,520 calls at buckets 1/8/128.
It excludes population and host setup; a small common boundary-probe cost remains.

4-KiB chunks (1-MiB declared maximum), 256 rows, uniform batch 32, round 0:

| Scope / checkpoint | Storage | ICSS, eight-page buckets |
| --- | ---: | ---: |
| Inner loop after Set | 3,040,020 | 3,042,082 |
| Inner loop after Push | 3,040,020 | 2,882,183 |
| Full handler after Set | 24,337,272 | 24,418,902 |
| Full handler after Push | 24,337,272 | 24,419,160 |

The apparent 159,899-instruction improvement disappears at the handler boundary
(+258 instead). At buckets 1/128 the corresponding ICSS handler counts remain
24,418,970/24,418,923 across growth. The large synthetic reply dominates total
cost. This does not justify tuning geometry or claiming a faster Vector algorithm.

Selected `vector-execution-boundary` CSV rows retain both scopes; raw artifacts
are under `target/evidence/storage-icss-parity/` in `read-trace/`,
`allocator-access/` and `execution-boundary/`. Reproduce via
[the benchmark guide](../../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md#iterator-coverage).

## Step 8: remove small fixed Vector read allocations

[BaseVec](../../../crates/ic-storage/src/engine/base_vec.rs) uses a separate stack
helper for direct Get/Pop of fixed-width values ≤32 bytes without destructors.
Other values keep owned decoder input; iterators keep their reusable buffer.
The initial inline trial regressed misses by up to 1.2%; isolating the helper
removes that regression. No API, format or allocation-policy change is required.

256 rows, uniform batch 32, round-0 GetAfterSet, eight-page ICSS buckets:

| Type / scope | Storage before | Storage final | ICSS 0.7.2 |
| --- | ---: | ---: | ---: |
| u64 / loop | 101,432 | 95,032 | 102,933 |
| Fixed 32 / loop | 104,970 | 98,570 | 104,577 |
| u64 / full handler | 670,749 | 664,349 | 752,611 |
| Fixed 32 / full handler | 796,572 | 790,172 | 876,746 |

The same 6,400 saving survives outside the loop: 6.1–6.3% loop savings become
0.8–0.95% full-handler savings. Temporary allocations fall from 32 to zero versus
ICSS's 32. Fixed widths 33/64/256, variable chunks and Wire controls stay unchanged.
Of the 15-type/13-operation cells, 84 u128 cells rise six instructions; others
improve or match. The benchmark Wasm shrinks 3,735 bytes.
Final qualification checks 7,200 calls: 4,680 typed, 840 ordinary, 840 allocation
probe and 840 boundary probe; all boundary inner counts match ordinary results.
`vector-fixed-*` CSV rows retain selected before/trial/final matrices. Raw evidence
is `target/evidence/storage-icss-parity/vector-inline/`; final build is `mxipc1`.

## Qualification, reproduction and delivery boundary

The final implementation passed 72 native tests, strict core Clippy and installed
original/O3/Os/Oz IO, custody, 1,031-allocation upgrades and trap rollback in
`target/qualification/runtime/attempt.XBXLTP`. The benchmark passes 38 focused
library/driver tests, including checkpoint/trace admission, strict Clippy and formatting.
Run `BENCH_WORKLOAD=depth make benchmark-storage` or `BENCH_WORKLOAD=vector-reads
make benchmark-storage`; select `BENCH_UPSTREAM_BUCKET_PAGES=1`, `8` or `128`.
Qualification uses Rust 1.99.1, Testkit 0.79.0 and PocketIC 16.1.0. It explicitly
patches the fixed Runtime package from Platform `998764c`, labelled 0.19.2 but
**not published 0.19.2**. Platform deferred its export in `3ad71e6`; source remains
at `refs/retained/runtime-stable-read-uninit-20261106`. Do not reapply over its release.
Runtime 0.19.3 lacks the export; subsequent adoption of published 0.19.4 removes
that blocker. Current registry/lifecycle proof is in the handoff, separate from these trials.
Normal locks are restored; concurrent Logging 0.24.9/Metrics 0.29.4/Runtime 0.19.3/Wire 0.17.4 selections are not qualified here.
Overrides, locks and failures remain under `target/evidence/storage-icss-parity/`.
