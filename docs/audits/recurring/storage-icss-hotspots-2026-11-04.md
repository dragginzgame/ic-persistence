# Storage/ICSS hotspot and allocation investigation — 2026-11-04

## Follow-through: encoded inline-value replacement

The RSX review identified packed encoded mutation as a transferable idea:
[Hylix map at 757bf506](https://github.com/Neutrinomic/cdk-rsx/blob/757bf506bb84c6487d6b367d56bb7f7bed96aea3/ver1/crates/cdk-rsx-collections/src/hylix/map.rs).
This experiment implements a narrower Storage change: an existing inline value
replaced by another inline value copies unchanged encoded spans into one final
node frame, without constructing separate buffers for every key and inline value.
The common record writer still seals the complete payload checksum. The existing
allocation, staged publication, cache invalidation and old-value decode refusal
paths remain. External transitions, insertion and rebalancing retain their paths.
No public API or stored format changes; no RSX dependency is introduced.

Matched installed runs use 256 rows, shuffled population, Uniform/Hot16 access,
counts 1/32, three stateful rounds, one sample and disabled caches. Both traces
pass 3,576 measured backend calls and final-content validation. The 596 aggregate
rows retain identical ICSS instruction counts and Storage stable-page counts.
ICSS is published DFINITY ic-stable-structures 0.7.2 with eight-page buckets.

Selected Uniform Replace medians, in instructions:

| Case | Count | Before | Candidate | Reduction |
| --- | ---: | ---: | ---: | ---: |
| u64/u64 | 1 | 198,145 | 186,718 | 5.77% |
| u64/u64 | 32 | 1,379,545 | 991,852 | 28.10% |
| 32-byte array keys/values | 1 | 215,206 | 202,242 | 6.02% |
| 32-byte array keys/values | 32 | 1,897,870 | 1,487,289 | 21.63% |
| u64/32-byte Vec | 32 | 1,585,676 | 1,184,110 | 25.32% |

Across the selected inline Replace rows, reductions are 3.83–7.74% for one update
and 20.21–28.10% for 32 updates. This is not a 2x or whole-endpoint claim.
External Replace rows change by less than 0.04% in either direction. The largest
aggregate regression is U64Bytes/33, Hot16, count32 Last: 785,477 to 806,360
(+2.66%); its cause is not isolated. Other regression rows remain visible in the
complete CSV. No geometry or checksum change is credited for these results.

The uncompressed comparison Wasm grows from 1,023,670 to 1,032,987 bytes (+9,317).
Its section-10 code payload grows from 896,856 to 905,925 bytes (+9,069), measured
by parsing Wasm section lengths. The existing comparison recipe uses opt-level 3,
fat LTO, one codegen unit and no Binaryen pass; these are collection measurements.

Retain this candidate: the bounded local change has meaningful update savings,
with modest measured size and unrelated-operation tradeoffs. Ninety-five core
tests pass, including byte-for-byte equivalence with the ordinary encoder for
every inline length and entry position in leaf/branch nodes, mixed external
descriptors, and existing corruption, refusal, cache and reopening histories.
Strict all-target core Clippy passes after correcting a collapsible-if lint.
Both incoming lockfiles remain byte-identical. Evidence:
[comparison](../../measurements/storage-encoded-update-comparison-2026-11-04.csv),
[configuration and identities](../../measurements/storage-encoded-update-investigation-2026-11-04.json),
`target/encoded-update-investigation/` and matched attempts `BnaSBi` / `6yIcYH`.

### Selected smaller implementation

A follow-up reuses existing old-value and replacement-value preparation before
the encoded replacement, reducing code duplicated across typed tree instances.
Preventing helper inlining alone produced identical module/code sizes and was
reverted. The shared-path candidate is selected despite modest extra per-value
work compared with the first fast path. It retains the full-node allocation
avoidance, stored format and publication/error contracts.

| Uncompressed comparison artifact | Original | First candidate | Selected smaller candidate |
| --- | ---: | ---: | ---: |
| Complete module bytes | 1,023,670 | 1,032,987 | 1,027,724 |
| Code-section payload bytes | 896,856 | 905,925 | 900,749 |

This removes 5,263 module bytes and 5,176 code bytes from the first candidate,
leaving increases of 4,054 / 3,893 bytes versus the original. The compiler,
comparison profile and section-length measurement method are unchanged.
The same 3,576-call trace passes in `attempt.fDd0Gy`; all 596 aggregate ICSS
instruction controls and Storage stable-page counts match the original.
Selected inline Replace reductions versus the original are 3.74–7.47% for one
update and 19.53–26.60% for batches of 32. These rows cost up to 2.39% more than
the first candidate. The largest unrelated regression remains Last +2.66%.
Ninety-five core tests and strict all-target Clippy pass. Lockfiles are unchanged.
The [three-way comparison](../../measurements/storage-encoded-update-size-2026-11-04.csv)
and the identity JSON above retain the result; build/test logs and the rejected
no-inline artifact remain under `target/encoded-update-size/`.

## Experiment 2: smaller external-value blocks (research only)

The selected smaller encoded-update implementation remains in production source.
Two temporary allocator prototypes were measured and then reverted: a global
64-byte block floor, and a 64-byte external-value floor with routing nodes still
using 256 bytes. Both used experimental `ICMAPDNS` admission, not `ICMAP002`.
Current readers reject sub-256-byte pointers; a production change would therefore
need an explicit format transition and host admission/rebuild plan. The first
test run caught exactly this consumer framing constraint; the prototype's region
fixture was explicitly changed for qualification and restored afterward.

Native accounting uses four governed maps on one backing, each with 1,024 rows,
nine payload sizes and six half-delete/reinsert rounds. Counts include directory
metadata, allocation growth and free-list writes. Every displaced and final value
is checked. Both prototypes give the following populated physical page counts:

| Payload bytes | Current 256-byte floor | Prototype | Capacity reduction |
| --- | ---: | ---: | ---: |
| 33–44 | 27 | 15 | 44.44% |
| 45–108 | 27 | 19 | 29.63% |
| 109–236 | 27 | 27 | 0% |
| 237 | 43 | 43 | 0% |

Physical pages are 65,536 bytes. These are complete backing capacities, not just
value slots; individual 33-byte values plus their 20-byte records fit 64 bytes.
Capacity stays at these levels throughout the six reuse rounds. This does not
prove arbitrary size-churn convergence or coalescing. The value-only prototype's
population writes for 33-byte rows increase from 1,783,188 to 1,826,904 bytes
(2.45%); denser geometry is not free of allocation/publication work. At 109 bytes
and above its native population writes match the baseline exactly.

Installed comparisons use the same 256-row Uniform/Hot16, one-sample, three-round
trace as experiment 1. Each prototype passes 3,576 backend calls and final-content
validation; all 596 aggregate ICSS instruction controls match the baseline.
ICSS remains published DFINITY 0.7.2. For the narrower value-only prototype,
selected 33-byte, 32-operation aggregate medians are:

| Operation | Pattern | Current instructions | Prototype | Change |
| --- | --- | ---: | ---: | ---: |
| Get | Uniform | 774,436 | 697,324 | -9.96% |
| Replace | Uniform | 1,135,044 | 973,989 | -14.19% |
| Scan | Uniform | 268,756 | 189,504 | -29.49% |
| Get | Hot16 | 808,162 | 733,585 | -9.23% |
| Replace | Hot16 | 1,171,365 | 1,013,332 | -13.49% |
| Scan | Hot16 | 267,637 | 187,950 | -29.77% |
| Grow | Uniform | 1,184,610 | 1,668,235 | +40.83% |
| Grow | Hot16 | 1,254,035 | 1,408,555 | +12.32% |

The actual Grow operation changes 33-byte payloads to 132 bytes. Current blocks
can retain their address; 64-byte blocks cannot. Smaller blocks trade growth
headroom for density. First and KeysScan also have small regressions; full rows
remain in the CSV. Some large reductions are near a page-charge increment, but
no fault trace was captured, so placement attribution remains a hypothesis.
Stateful round medians are not fresh-population measurements for every operation.

The global-floor prototype also increases selected 1-KiB single scans by about
29%. Keeping routing nodes at 256 bytes avoids those large unrelated regressions
in the measured trace, without losing the small-value capacity savings. The
value-only comparison artifact is 1,028,195 bytes, with a 901,220-byte code payload:
both increase 471 bytes over the selected experiment-1 baseline. The global
prototype increases each by three bytes. Sizes are uncompressed; section-10
payload lengths and the unchanged comparison compiler/profile define the measure.

Decision: retain the value-only prototype as research, not a production default.
It is promising for stable-size small records, but growth regressions, consumer
format migration, warm-cache behavior and real consumer distributions must be
resolved before promotion. No slab allocator, public tuning option, compatibility
reader or automatic reset is introduced. Both prototypes pass 95 core tests,
including corruption/reopening/variable-size histories, and strict core Clippy.
The native helper passes strict Clippy. Source restoration and lockfile checks
preserve the selected 0.66.3 implementation; no release is requested.

Evidence: [native capacity/IO](../../measurements/storage-small-block-native-2026-11-04.csv),
[installed comparisons](../../measurements/storage-small-block-comparison-2026-11-04.csv),
[identities and limits](../../measurements/storage-small-block-investigation-2026-11-04.json),
[global prototype patch](../../measurements/storage-small-block-global-prototype-2026-11-04.patch),
[value-only prototype patch](../../measurements/storage-small-block-values-prototype-2026-11-04.patch).
Patches apply to the experiment-1 source identified in the metadata and are not
production changes. Raw evidence remains under `target/small-block-investigation/`
and installed attempts `LueDBl` / `CeZacI`; baseline is `fDd0Gy`.

## Deeper allocation investigation

This follow-up compares the accepted experiment-1 source with three research
policies. All keep the node allocation floor at 256 bytes, preserve checksums,
returned-value validation, staged writes and governance, and use fresh images:

| Policy | New external-value minimum | Replacement behavior |
| --- | ---: | --- |
| Baseline | 256 B | Existing fourfold-shrink/reusable-block rule |
| `value64` | 64 B | Same shrink rule with the smaller minimum |
| `value128` | 128 B | Same shrink rule with the smaller minimum |
| `retained64` | 64 B | Use `min(previous_capacity, 256)` as the replacement floor; otherwise the existing shrink rule |

The last policy uses the existing descriptor, with no new per-value history,
allocation table, API option or coalescing algorithm. The floor is **not** a
256-byte capacity cap. Larger blocks still follow the existing fourfold rule;
external-to-inline replacement frees the block and loses its capacity history.
All three research formats have distinct markers. None is deployed or selected
as the production format. These are B-tree value updates, not Cell schema changes.

### Work and measurement boundaries

The native workload has four governed maps of 1,024 records, nine histories,
24 phases each, six cold reopen/full-content checks, full deletion and small
refill. Histories cover fixed 33/80-byte values, 33↔132 and 80↔320 alternation,
one-time growth, 10%/50% growth, staircase sizes through 4,096 bytes, and inline
transitions. Mutation returns are checked against an independent map. Counters
include physical backing and directory IO; diagnostic and verification reads
are excluded from the next mutation phase. Native IO is not an IC instruction
estimate. Final refill uses sorted keys, unlike the shuffled initial population;
its capacity difference cannot alone be attributed to fragmentation or leakage.

Each installed policy uses two shuffle/access seeds, batch sizes 1/32, published
ICSS 0.7.2 with eight-page buckets, and the same six configurations:

| Configuration | Population | Payloads | Rounds | Access | Storage cache |
| --- | ---: | --- | ---: | --- | ---: |
| Boundaries | 1,024 | 33,44,45,108,109,132 | 3 | Uniform | Disabled |
| Warm mixed | 4,096 | 33,80,132 | 3 | Uniform/Hot16 | 64 KiB |
| Large cold/warm | 16,384 | Fixed 33-byte array | 3 | Uniform/Hot16 | Disabled / 64 KiB |
| Cycles cold/warm | 4,096 | 33,80 | 12 | Uniform/Hot16 | Disabled / 64 KiB |

The ordinary configurations retain the complete supported operation sequence.
Cycles deliberately repeat SmallReplace → SmallGet → Grow → LargeGet on the
same keys, with no remove/reinsert or empty-value phase; Grow uses four times
the base size here. Each configuration starts fresh. Setup is outside timing,
but its capacity remains live. Results and final contents are checked on both
backends. Cache-enabled Storage and published ICSS have different cache
capabilities; this is not an equal-cache-budget comparison.

Per-round CSV values are upper medians of two samples, not percentile estimates.
Whole-canister stable pages are 64 KiB high-water counts, including controls and
unused capacity. Collection instructions exclude request preparation, codec
responses and validation. These are not whole-endpoint or production traffic
measurements. Sums across different workload families are not a universal score.

### Verified native tradeoffs

Physical pages after phase 23 (all four maps together):

| History | Baseline | 64 B | 128 B | 64 B retaining capacity |
| --- | ---: | ---: | ---: | ---: |
| Fixed33 | 23 | 11 | 15 | 11 |
| Fixed80 | 23 | 15 | 15 | 15 |
| Toggle33 | 23 | 27 | 31 | 27 |
| Toggle80 | 55 | 47 | 47 | 47 |
| GrowOnce | 23 | 27 | 31 | 27 |
| Mixed10 | 23 | 15 | 19 | 15 |
| Mixed50 | 23 | 19 | 23 | 19 |
| Staircase | 535 | 547 | 543 | 547 |

Total physical write bytes over all 24 phases, including population, in decimal MB:

| History | Baseline | 64 B | 128 B | 64 B retaining capacity |
| --- | ---: | ---: | ---: | ---: |
| Toggle33 | 11.453 | 42.332 | 12.787 | 12.799 |
| Toggle80 | 24.338 | 53.884 | 53.884 | 24.342 |
| GrowOnce | 6.992 | 9.681 | 8.327 | 8.338 |
| Mixed10 | 7.076 | 10.191 | 7.215 | 7.227 |
| Mixed50 | 9.019 | 24.486 | 9.690 | 9.702 |

Capacity retention removes the repeated small-class relocation in the selected
oscillating histories. It does not avoid the first relocation or return high-water
memory to the system. The allocator splits reusable larger blocks but does not
coalesce smaller ones; freed small blocks can remain available while a growing
value needs new larger capacity. This is reusable capacity, not proof of a leak.
No general plateau claim follows from 24 finite phases.

The 128-byte floor avoids the fourfold 256→64 shrink, but 512→128 still qualifies.
That explains why it helps 33↔132 and leaves 80↔320 churn. Retaining a replacement
floor up to 256 avoids both, while keeping new 33-byte values in 64-byte blocks.
The 20-byte checked frame is part of these sizes: 44 bytes fits 64; 45 requires
128; 109 requires 256. The 32-byte inline threshold is unchanged.

### Installed results and recommendation

All 24 runs pass 39,168 measured backend calls and final-content validation.
Complete normalized raw ICSS responses, instructions and capacity match across
policies for each configuration, not merely their medians. Thirty-six native
histories complete. The 64/128-byte candidates pass 95 core tests each; retention
passes 96, including repeated reopen with unchanged grown-value identity. Strict
core and harness Clippy pass; focused harness tests pass (6 library, 10 driver,
1 native-history example). No production upgrade/release qualification is claimed.

Selected fixed-array results: 16,384 rows, 33-byte values, Uniform, count 32,
cold cache, round zero. Both 64-byte policies have the same Get instruction count;
the retention candidate's replacement count includes its extra capacity check.

| Measure | Baseline | Retaining 64 B | ICSS 0.7.2 |
| --- | ---: | ---: | ---: |
| Stable pages after Get | 81 | 33 | 25 |
| Get instructions | 5,959,443 | 3,469,706 | 2,374,743 |
| Replace instructions | 8,466,923 | 5,341,182 | 4,313,415 |

The candidate closes much of the density gap but does **not** beat ICSS in these
selected rows. A 59% page reduction is not a 59% instruction reduction.

Retaining 64-byte policy versus baseline, **complete twelve-cycle traces**:
upper median of the two per-sample instruction totals, not a sum of per-phase
medians. Negative means fewer instructions. These include all four cycle phases.

| Base bytes | Count | Access | Cache disabled | Cache 64 KiB |
| --- | ---: | --- | ---: | ---: |
| 33 | 1 | Uniform | -17.68% | +9.27% |
| 33 | 1 | Hot16 | -15.37% | -33.32% |
| 33 | 32 | Uniform | -47.57% | -53.86% |
| 33 | 32 | Hot16 | -43.35% | -76.07% |
| 80 | 1 | Uniform | -0.04% | +61.10% |
| 80 | 1 | Hot16 | +0.02% | -34.93% |
| 80 | 32 | Uniform | -19.01% | -16.49% |
| 80 | 32 | Hot16 | -1.98% | -23.87% |

Retention fixes the clearest recurring allocation churn, but does **not** justify
changing the default for the user's single-record target: the warm Uniform
80-byte trace regresses **61.10%**, and the 33-byte trace regresses **9.27%**.
Individual first-growth calls also regress; the warm single Uniform 33-byte
first Grow is +292.36%, while the cold counterpart varies by selected key.
The ordinary boundary matrix includes a +50.90% single Grow regression at 45
bytes in round two. Large percentage changes are shown alongside absolute counts
in the retained per-round data; no universal throughput claim follows.

Recommendation: preserve the accepted encoded inline-update optimization and
retain capacity-aware 64-byte allocation as the best **research candidate**,
not a production change. Reject the plain 64-byte minimum as an unconditional
replacement; the 128-byte floor does not solve both recurring-growth cases.
Next isolate cache collisions and tracked-memory costs on single-record paths,
then measure real consumer encoded-size/resize distributions. Installed concurrent
map populations and explicit fresh-image/upgrade lifecycle qualification remain
necessary before choosing a format transition. Do not add a slab allocator,
coalescer, public sizing menu or migration compatibility layer from this evidence.

The matched comparison Wasm is 1,027,750 bytes at baseline, 1,028,221 for 64 B,
1,028,218 for 128 B, and 1,028,347 for retention. Section-10 code payloads are
900,775 / 901,246 / 901,243 / 901,372 bytes. Retention adds 597 module bytes to this
baseline. The expanded harness differs from the earlier experiment's build;
these are matched sizes within this study, not a change to the accepted encoder.

### Cache sensitivity and consumer scope

Verified implementation: the direct-mapped routing cache indexes a node using
`((offset >> 8) ^ (offset >> 17)) % slots`. Smaller external blocks change node
addresses even when node sizes and logical tree shape are unchanged. Verified
measurements show large cache-enabled swings: for the 33-byte, single-operation,
Uniform case in round 1, Contains changes from 3,276 instructions to 175,054 for
the 64-byte layout. Another round improves. At 132-byte payloads the value floor
no longer changes geometry and this selected Contains comparison is unchanged.
The precise share caused by cache collisions versus tracked-memory effects is
**not isolated** by these results. Direct hit/miss and touched-region attribution
is the next focused diagnostic, not a reason to claim an allocator-only speedup
or immediately replace the cache. Even cache-disabled key-only scans can regress
when the physical node addresses change.

Actual consumers were checked read-only. Canic's
`crates/canic-core/src/storage/stable/cycles.rs` uses a variable-size, bounded
64-byte `CyclesFundingLedgerRecord`; `CycleTrackerBalanceRecord` serializes a
fixed 16-byte balance and already fits inline. Its placement-index record has
variable Pending/Bound forms under a 192-byte bound. Blob's tenant enrollment
record includes application framing around its fields. These declarations do
not establish actual encoded-size or resize distributions. No numeric consumer
saving is claimed, and no sibling code or release was changed. The
[consumer inventory](../../local-memory-audit.md) remains a declaration inventory.

### Reproduction and retained evidence

The benchmark README documents `BENCH_PAYLOADS`, `BENCH_GROWTH_CYCLES` and
`--allocation-histories`. Use the configuration matrix above with two samples,
shuffling and eight-page ICSS buckets; every resolved setting, compiler, source
manifest and Wasm digest is in the metadata below. Raw traces/source archives and
qualification logs remain under `target/storage-comparison/` and
`target/small-block-deep/`. The production tree and both incoming locks were
restored/verified byte-for-byte; accepted experiment 1 and unrelated work remain.

- [Per-round installed results](../../measurements/storage-small-block-deep-installed-2026-11-04.csv)
- [Complete cycle totals](../../measurements/storage-small-block-deep-cycle-totals-2026-11-04.csv)
- [Native histories, metadata and IO](../../measurements/storage-small-block-deep-native-2026-11-04.csv)
- [Run identities, validation and limits](../../measurements/storage-small-block-deep-2026-11-04.json)
- [128-byte research patch](../../measurements/storage-small-block-value128-prototype-2026-11-04.patch)
- [Capacity-retention research patch](../../measurements/storage-small-block-retained64-prototype-2026-11-04.patch)

The earlier value-only patch supplies the 64-byte control. Patches apply to the
accepted experiment-1 production source identified by hashes in the metadata;
apply only one at a time. They deliberately include fresh-format fixture updates.

## Cache-placement attribution: smaller-block regression

The 61.10% warm single-record regression is reproducible and sensitive to the
node cache's direct-mapped slot selection. Four uninstrumented matched runs vary
only allocator geometry (baseline or retained64) and cache hash (existing or a
64-bit mixer). Four separate diagnostic runs enable the existing cache counters
in Wasm and query their deltas after each operation. Diagnostic instruction
counts are excluded: instrumentation changes the program and heap layout.

All runs use 4,096 shuffled u64/bytes records, payloads 33 and 80, Uniform/Hot16,
counts 1/32, two seeds, twelve four-phase small/large growth cycles and a 64 KiB
cache. Comparisons use published DFINITY **ic-stable-structures 0.7.2**, eight-page
buckets. The original baseline module and summary reproduce the earlier study
exactly. This is a cache/address diagnostic, not a new allocator qualification.

Complete 48-call, Uniform, count-one, 80-byte cycle totals (uninstrumented):

| Allocator / cache hash | Seed 1 instructions | Seed 2 instructions |
| --- | ---: | ---: |
| Baseline / original | 7,009,246 | 6,810,252 |
| Baseline / mixer | 11,115,867 | 6,807,172 |
| Retained64 / original | 11,291,880 | 6,808,118 |
| Retained64 / mixer | 7,012,181 | 6,894,638 |

The mixer removes the selected retained64 slowdown but introduces a 58.59%
baseline slowdown. The second seed remains near baseline. A two-sample upper
median therefore conceals that this large regression is specific to one history;
the retained evidence reports each sample independently.

The existing hash admits only one node per slot. It protects depth-zero/one nodes
against deeper-node replacement. In the original-hash diagnostic's final round,
baseline seed 1 records four hits per call; retained64 records three hits, one
collision miss and one protected rejection on every phase. Retained64 uses
18,365 bytes of the 65,536-byte budget, with zero budget rejections. Its colliding
node cannot become warm even though most of the cache budget is unused.
The second seed has three hits and no misses for both allocators.
The mixer reverses the seed-one outcome: retained64 has four hits and no misses;
baseline has three hits, one collision miss and one protected rejection per call,
using only 13,778 bytes. All eight installed runs pass response/final-content
validation (12,288 benchmark calls, half instrumented). Every uninstrumented
ICSS response matches exactly across policies; hash-only changes preserve every
Storage stable-page count.

This verifies a cache admission pathology, rather than evidence that smaller
blocks inherently cost 61% more. The extra backing read, record validation and
allocation may incur both ordinary instructions and memory tracking charges.
These experiments do not count actual accessed/dirty regions and cannot assign
an exact fraction of the slowdown to page charges.

Do not adopt the mixer: it merely changes which paths collide and fails the
existing `routing_cache_keeps_repeated_right_edge_misses_in_heap` performance
regression test (64 backing reads rather than zero). Baseline/mixer has 94 passing
core tests; retained64/mixer has 95; each has this one failure. Both strict library
Clippy checks pass. The baseline diagnostic passes all 95 core tests. Installed
runs still verify responses and final logical contents; performance-test failure
is retained, not treated as qualification success.

The next justified experiment is bounded collision handling (for example, a
small set-associative cache), retaining upper-node preference and the actual
heap-byte limit. Qualify multiple address histories, right-edge misses, batches
and cold-cache workloads. A larger default cache or another hash alone is not
established as a fix. Keep the allocator and production cache unchanged until
that experiment proves a benefit without moving this failure to another path.

Evidence: [per-call timings](../../measurements/storage-cache-timing-calls-2026-11-04.csv),
[per-sample totals](../../measurements/storage-cache-timing-totals-2026-11-04.csv),
[diagnostic counters](../../measurements/storage-cache-diagnostics-2026-11-04.csv),
[configuration and module/source digests](../../measurements/storage-cache-investigation-2026-11-04.json),
[hash prototype](../../measurements/storage-cache-mixed-prototype-2026-11-04.patch)
and [counter/query prototype](../../measurements/storage-cache-diagnostic-prototype-2026-11-04.patch).
The diagnostic patch exposes a temporary research-only method and query; neither
is a production API proposal. Apply it to the incoming accepted baseline, then
optionally apply the hash/retained64 research patches. Reproduce with
`BENCH_CASES=U64Bytes BENCH_PAYLOADS=33,80 BENCH_PATTERNS=uniform,hot16 BENCH_SHUFFLED=1 BENCH_ROUNDS=12 BENCH_GROWTH_CYCLES=1 BENCH_CACHE_BYTES=65536 make benchmark-storage BENCH_ROWS=4096 BENCH_SAMPLES=2`.
Raw logs, sources and responses remain under `target/cache-placement/` and the
recorded `target/storage-comparison/` attempts. Temporary source changes are
restored after the experiment, preserving accepted optimizations and CI work.

## Two-way cache experiment and the ICSS comparison

This follow-up preserves the current address mixer, upper-node preference and
heap-byte admission budget, but permits two nodes per set. The first way is most
recently used; hit/insert swaps establish recency without timestamps or another
table. Replacement selects an empty way, otherwise the least-recent eligible
way; deeper nodes still cannot evict depth-zero/one nodes. Invalidation searches
both ways. The approximate slot budget is rounded down to an even count: the
64 KiB Wasm cache still has 62 slots, now in 31 sets. Very small budgets that
cannot admit two slots disable this research cache. No backing format or public
signature changes are proposed.

The hash is still `((offset >> 8) ^ (offset >> 17))`, reduced modulo the number
of sets. The rejected two-multiplication mixer is not used. Associativity changes
collision handling rather than making the hash more random. This is bounded to
two probes, but does not prevent three competing nodes sharing a set.

Matched twelve-cycle, 4,096-record U64Bytes runs repeat the previous 33/80-byte,
Uniform/Hot16, count-one/32, two-seed matrix. Each complete per-sample trace has
48 calls. Installed timings are uninstrumented; diagnostics are separate.

| Uniform, 80-byte, count-one trace | Original direct map | Two-way | Change |
| --- | ---: | ---: | ---: |
| Baseline allocator, seed 1 | 7,009,246 | 6,934,452 | −1.07% |
| Baseline allocator, seed 2 | 6,810,252 | 6,819,420 | +0.13% |
| Retained64 allocator, seed 1 | 11,291,880 | 7,027,774 | −37.76% |
| Retained64 allocator, seed 2 | 6,808,118 | 6,821,596 | +0.20% |

The selected retained64 pathology falls back near the original baseline (+0.26%
relative to baseline/direct). Across all selected complete warm traces, baseline
allocator changes range from −26.89% to +1.20%; retained64 changes range from
−37.76% to +2.04%. Hot16 batch traces generally pay additional overhead. These
are per-history outcomes, not a weighted application throughput estimate.

The separate retained64 diagnostic confirms four hits, zero collision misses and
zero protected/budget rejections on every phase of seed one's final count-one
80-byte Uniform round; the cache uses 26,239 bytes of its 65,536-byte budget.
This contrasts with three hits plus a protected collision on the direct cache.
Five installed runs validate 9,408 benchmark calls (7,872 uninstrumented and
1,536 diagnostic), including final contents. Matched ICSS raw responses and
Storage physical stable-page counts are identical.

### What DFINITY's ICSS actually does

Two different ICSS sources must not be conflated:

- Published **ic-stable-structures 0.7.2**, the installed comparison backend,
  has no persistent decoded-node cache. `BTreeMap::traverse` calls `load_node`,
  which calls `Node::load`. A node can lazily cache its own decoded entries during
  its lifetime; that is not a cross-call address-indexed node cache.
  Its memory manager's `BucketCache` holds one virtual-segment/address mapping
  and performs a containment check; it has no hash table and does not cache node
  contents. See [published B-tree source](https://github.com/dfinity/stable-structures/blob/v0.7.2/src/btreemap.rs)
  and [bucket cache](https://github.com/dfinity/stable-structures/blob/v0.7.2/src/memory_manager.rs).
- The already pinned DFINITY Git research control
  **`b5d1650463f477affb3d358e86b1113953fc9937`** does have a direct-mapped node
  cache. Its index is `(address / page_size) % slots`; the default is 16 slots.
  `page_size` here is the B-tree node allocation geometry, not the 64 KiB Wasm
  page. Depth-zero/one nodes have priority, and collisions among deeper nodes
  use last-writer replacement. Capacity is selected by slot count, not Storage's
  accounting of actual retained node-buffer heap bytes. Nodes are taken from and
  returned to the cache around traversal, whereas Storage shares immutable read
  nodes through `Rc`. See the [pinned cache implementation](https://github.com/dfinity/stable-structures/blob/b5d1650463f477affb3d358e86b1113953fc9937/src/btreemap/node_cache.rs)
  and [map defaults](https://github.com/dfinity/stable-structures/blob/b5d1650463f477affb3d358e86b1113953fc9937/src/btreemap.rs).
  These details were verified in Cargo's locked local Git source; they are not a
  claim about current GitHub HEAD or a later release.

The pinned Git implementation shares the direct-mapped collision limitation.
That does not establish identical miss rates: node addresses, allocation sizes,
tree geometry and slot counts differ. This experiment does not measure two-way
Storage against the pinned Git node cache at an equal memory budget. Its timing
control is published ICSS 0.7.2, whose complete responses are verified unchanged.
Neither hash is cryptographic; both are cheap physical-address slot selectors.
Storage's checksum/record integrity is separate and unchanged.

### Qualification and decision

Baseline/two-way passes all 96 core tests and strict library Clippy; retained64/
two-way passes 97 core tests, including the extra retention test. Coverage checks
colliding ancestor/leaf residency, refusal when both ways contain protected
ancestors, recency replacement, invalidation of either way, exact accounting and
budget rejection without losing admitted neighbours. The existing right-edge
zero-backing-read regression passes; the rejected mixed hash failed it.

Two additional matched runs cover U64U64 Map and u64 Set, 4,096 shuffled rows,
counts 1/32, Uniform/Hot16/Zipf/sequential access, two seeds and three rounds of
the ordinary operation sequence, with the cache reset before every call.
These runs include misses, scans, insertions and removals instead of only repeated
value-growth cycles. Complete traces range from −0.60% to +1.50%. The largest
individual non-control regression is Map First +4.25% (195,644 → 203,967).
The tiny Control row changes 423 → 478 instructions (+13.0%, only 55 instructions);
it remains in the raw data. Largest individual warm regressions are +3.17% for
the baseline allocator and +3.82% for retained64, both batch SmallGet.

**Adoption update:** the maintainer subsequently selected the two-way cache for
pending Storage 0.66.3. The measured implementation is applied with the existing
allocator and original address hash; no diagnostic API is retained. Native tiny-
budget coverage now spans disabled and enabled two-way configurations. The
retained64 allocator and stronger hash remain research-only. No public API or
persisted-format change is needed, and no release has been executed.
Adoption revalidation passes all 96 core tests, strict library Clippy, focused
rustfmt, documentation links and whitespace checks. The cache source is byte-for-
byte identical to the timed prototype. All-target/test-target Clippy is blocked
by an unrelated `format_collect` warning in the pending repo-tool
`tests/ci_validation.rs:9`; that file was preserved during cache adoption. The subsequent authorized release
fixes its string construction; all-target Clippy and 87 helper tests pass. Logs are
under `target/cache-associative/adoption/` and `target/storage-0663-deploy/`.

The measured tradeoffs remain: two slots can still lose against three competing
nodes, and only two population histories and one 64 KiB installed cache budget
were timed. A wider seed/budget/key-value sweep remains useful follow-up, not
completed qualification. No universal improvement over ICSS is established.

[Warm per-sample totals](../../measurements/storage-cache-two-way-totals-2026-11-04.csv),
[cold per-sample totals](../../measurements/storage-cache-two-way-cold-totals-2026-11-04.csv),
[all uninstrumented calls](../../measurements/storage-cache-two-way-calls-2026-11-04.csv),
[diagnostic counters](../../measurements/storage-cache-two-way-diagnostics-2026-11-04.csv),
[configuration and digests](../../measurements/storage-cache-two-way-investigation-2026-11-04.json),
[cache/test patch](../../measurements/storage-cache-two-way-prototype-2026-11-04.patch)
and [diagnostic patch](../../measurements/storage-cache-two-way-diagnostic-prototype-2026-11-04.patch)
retain the experiment. Apply the diagnostic patch after the two-way and retained64
patches; counter timings are not performance results. The previous warm recipe
is unchanged. The cold recipe is
`BENCH_CASES=U64U64,SetU64 BENCH_PATTERNS=uniform,hot16,zipf,sequential BENCH_SHUFFLED=1 BENCH_ROUNDS=3 BENCH_CACHE_BYTES=65536 BENCH_COLD_CACHE=1 make benchmark-storage BENCH_ROWS=4096 BENCH_SAMPLES=2`.
Sources, logs and analysis scripts remain in `target/cache-associative/` and the
recorded comparison attempts. After the experiment, all production sources were
restored; the subsequent selected adoption applies only the cache/test patch,
with expanded tiny-budget coverage. Prior optimizations, pending CI work and
both lockfiles are preserved.

## Earlier investigation scope

Scope: current Storage source versus published DFINITY **ic-stable-structures
0.7.2**, not an older Storage release. This follows the boundary/sequential
[measurements](../../measurements/storage-icss-expansion-2026-11-04.json).
No production format, integrity policy or public API is changed by this investigation.

## Verified: Cell payload hashing is an additional linear pass

[Cell write](../../../crates/ic-storage/src/adapter/cell.rs) encodes through a
borrowed `Cow`, checks bounds, grows if needed, computes FNV over header and
payload, then writes payload and header. There is no intermediate encoded-payload
clone for `Vec<u8>`. ICSS 0.7.2 `src/cell.rs::flush_value` borrows encoding too,
but writes a magic/version/length header and payload without a payload checksum.

Matched boundary-run Replace measurements, in instructions:

| Logical bytes | Storage | ICSS | Extra Storage cost |
| --- | ---: | ---: | ---: |
| 236 | 163,185 | 161,237 | 1,948 |
| 4,076 | 197,719 | 165,077 | 32,642 |
| 65,516 | 910,679 | 226,517 | 684,162 |

Between 236 and 4,076 bytes, the difference increases 30,694 instructions for
3,840 bytes: **7.993 extra instructions/byte**. This is consistent with the
verified extra checksum pass; it is not an isolated checksum-only measurement.
The large-row total also includes IO, memory tracking and allocation effects.
The [earlier actual-Wasm inspection](storage-icss-2026-11-03.md) already found
LLVM unrolling FNV eight bytes at a time. Manual unrolling is not a new fix.
Changing the checksum algorithm changes retained digests and requires an explicit
format decision. Removing verification is not a compatible performance fix.

## Verified: Vector already uses ICSS's reusable-buffer approach

[Vector get and iter](../../../crates/ic-storage/src/adapter/vector.rs) differ
in buffer ownership: get decodes owned bytes; iter reuses one buffer and supplies
borrowed bytes. ICSS `src/base_vec.rs` uses the same ownership choices. Owned
byte-value decoding therefore needs a copy in both iterators. Recommending buffer
reuse would duplicate an optimization already present in both libraries.

Storage additionally verifies slot index, length and payload checksum before
calling the value codec. Variable-size slots require header admission before
allocating/reading payload. ICSS reads size and payload without that checksum.
Both integrations still perform memory translation. Storage's directory access
already has a last-span fast path; assuming a full-directory lookup per IO is wrong.

The bounded byte-vector control declares the same 1 MiB maximum for both engines.
Storage's stride includes a 12-byte slot header; ICSS uses a four-byte length for
this bound. Different offsets, heap buffers and tracked pages can affect costs.
The measured 32-read, 4 KiB batch uses 4,229,555 instructions for Storage versus
3,122,085 for ICSS. That alone does not identify how much is checksum, tracking,
initialization or translation. It does not justify a slot-format rewrite.

## Verified: Set already shares its ordered-map engine

[Set](../../../crates/ic-storage/src/adapter/set.rs) uses the same B-tree engine
as Map, as does ICSS `src/btreeset.rs`. Neither needs another independent set
implementation. Storage's iterator retains bounded node stacks and validates
routing records; it does not restart a root search for each key. Both set
iterators project map keys. Unit values have no external payload to skip, so
switching to a key-only traversal does not remove external-value reads here.

The sequential 32-key scan costs 146,138 instructions versus 120,694 for ICSS,
a 25,444-instruction difference. Node decoding/checksumming, allocation and key
handling are candidates, not independently attributed causes. String-key Set
membership favors Storage in the same run; a universal Set slowdown is not shown.
ICSS 0.7.2's Set iterator is not double-ended; keep reverse scans excluded rather
than silently emulating a different operation.

## Reporting defect corrected: round state was obscured by aggregation

The full trace intentionally reuses state across rounds and is checked against
an independent model. However, the aggregate CSV made initial-size labels easy
to mistake for the size present in every operation. Cell ends each round with
Shrink; subsequent Get operations read an empty value.

For Cell's 65,516-byte boundary case, raw round 0 Get costs 226,312 versus
226,166 instructions and returns 65,516 bytes. Rounds 1 and 2 cost 473 for both
engines and return zero bytes. The old aggregate median consequently reports
473; it is not a full-size cached-read measurement. Replace remains 910,679 for
Storage in every round, and ICSS stays about 226,500: its roughly 4x gap survives
this distinction. Grow's first round includes initial growth effects and must
also be separated from later rounds.

The driver now emits `round-summary.csv` alongside the preserved aggregate CSV.
It computes medians across samples within each round and retains zero-based round
numbers. A regression fixture proves initial 100/10 costs stay visible even when
the aggregate is 1/1. This is a reporting correction; measured work, execution
boundaries, canister operations and existing historical summaries are unchanged.

## Decision

No production optimization is promoted from these observations. The clearest
Cell cost buys a current integrity guarantee. Vector already has the proposed
buffer mechanism, and Set's remaining overhead has not been isolated sufficiently.
A future format experiment must price checksum alternatives and slot placement
with corruption, reopening and lifecycle qualification. A Set change should first
show reduced node/codec work under both scalar and string keys. Preserve current
checks while measuring, and keep first-round versus steady-state results separate.

## Qualification

Fourteen native benchmark tests and strict benchmark Clippy pass. The installed
Cell/Vector/Set run passed 1,044 measured backend calls and final-content checks.
All 522 per-round CSV rows were independently recomputed from raw responses.
[Round results](../../measurements/storage-icss-hotspot-rounds-2026-11-04.csv) and
[configuration/source digests](../../measurements/storage-icss-hotspot-rounds-2026-11-04.json)
retain the qualification. One sample, three rounds, 64 rows (Cell one), sequential
access, shuffled population, disabled Map caches and eight-page ICSS buckets.
The earlier run whose README changed during execution remains failed evidence;
the final fixed-source run passes all source/lock checks. Both incoming lockfiles
and all pre-existing comparison work are preserved.

## Follow-through: borrowed cursor keys

A subsequent focused investigation found a removable allocation in
`Tree::advance_path`: each cursor step copied encoded key bytes into a `Vec`,
then decoded the owned buffer. The checked node already retained those bytes.
The cursor now checks the key bound and decodes a borrowed slice, returning its
typed key with the same value descriptor. Scalar/array keys avoid that temporary
allocation; String codecs still need their owned decoded string. Node checksums,
codec bounds/errors, range ordering and mixed-direction traversal remain.
This supersedes the earlier no-production-change decision for this small cursor
path only. Cell and Vector formats remain unchanged.

A matched before/after run gives these instruction counts (32-item scan limit):

| Case / operation | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Set u64 / Scan | 146,138 | 133,796 | 8.45% |
| Set String / Scan | 194,214 | 193,250 | 0.50% |
| Map 32-byte array / Scan | 196,317 | 183,932 | 6.31% |
| Map 32-byte array / ReverseScan | 206,300 | 191,188 | 7.33% |
| Map 32-byte array / KeysScan | 197,267 | 181,839 | 7.82% |
| Map 32-byte array / BoundedScan | 161,142 | 154,582 | 4.07% |

All ICSS instruction rows matched exactly between runs, as did the unchanged
Vector control. Two non-scan Map rows increased: single Reinsert by 81 instructions
(0.04%) and RemoveMiss by 348 (0.34%). These operations do not use the changed
cursor; code generation/layout sensitivity is a hypothesis, not an isolated cause.
The scalar Set scan still uses 10.85% more instructions than ICSS's 120,694.
The patch narrows that gap; it does not eliminate it or establish a 2x speedup.

All 94 core unit tests and strict all-target core Clippy pass. The new regression
loads existing u64-format nodes through a codec that rejects owned input and
checks borrowed decoding across internal/leaf traversal, range bounds, alternating
directions, exhaustion and both cache settings. Both installed comparison runs
pass 984 measured calls each plus final contents and fixed-source/lock checks.
[Before/after results](../../measurements/storage-cursor-comparison-2026-11-04.csv)
and [digests/configuration](../../measurements/storage-cursor-investigation-2026-11-04.json)
retain the evidence.

## Vector control: declared slot geometry matters

An additional installed run passes 672 calls across fixed `[u8;32]` and the
existing 1 MiB-bounded `ChunkBytes` vector. Both store 32 logical bytes for the
matched rows, but the bound and fixed/variable encoding differ. Round 0 results:

| 32-item operation | Storage fixed 32 B | Storage max 1 MiB | ICSS fixed 32 B | ICSS max 1 MiB |
| --- | ---: | ---: | ---: | ---: |
| Get | 133,729 | 2,606,315 | 105,340 | 2,669,870 |
| Scan | 117,618 | 2,601,556 | 94,027 | 2,591,157 |
| Set | 189,217 | 5,155,859 | 176,122 | 5,235,323 |

After Get, physical stable pages are 4 versus 1,028 for Storage, and 9 versus
1,017 for ICSS. Both reserve fixed-stride capacity from the declared maximum;
this is not a Storage-only inefficiency. Density/encoding can dominate raw
payload size. This combined control does not isolate page charges from other
costs or justify reducing a real consumer's valid maximum. A tighter bound is
appropriate only when the application contract already guarantees it. Replacing
the vector format with indirect variable-sized slots would be a separate design
experiment, not a compatible fix inferred from these measurements.
[Per-round results](../../measurements/storage-vector-layout-2026-11-04.csv)
retain the complete control, including mutation rounds and other payload sizes.

## Additional follow-through: lazy inline values in key-only scans

Key-only traversal still constructed an owned inline value in `advance_path`
and then discarded it in `TreeIter::step`. The cursor now constructs an optional
value descriptor only when values are requested. Node checksum, inline bounds,
pointer admission and key/range checks remain. Ordinary entry scans preserve
value decoding and error behavior; no format or public signature changes.

A matched run reduces 32-key array-Map KeysScan from 181,839 to 167,727
instructions (7.76%). The original pre-cursor baseline was 197,267, so both
changes together reduce that row 14.98%. Full Map scan rows improve by about
0.1%; other Map operations are unchanged. Set scans, which have empty inline
values, increase 0.26% for String and 0.62% for u64 versus the previous candidate.
Final u64 Set scan is 134,628 versus the original 146,138 (7.87% fewer).
The small Set regression is retained as an explicit tradeoff for the key-scan
saving, not hidden by a combined average. ICSS and Vector controls match exactly.

All 94 core tests, strict all-target core Clippy and the 984-call installed
comparison pass, including final content and source/lock verification. Existing
key-only corruption/codec tests continue to exercise validation boundaries.
The new code adds no unsafe access or special unchecked read path.
[Results](../../measurements/storage-key-only-comparison-2026-11-04.csv) and
[configuration/digests](../../measurements/storage-key-only-investigation-2026-11-04.json)
retain this second change independently of the earlier cursor candidate.
