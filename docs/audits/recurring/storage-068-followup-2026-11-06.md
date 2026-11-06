# Storage 0.68 follow-through: matched ICSS comparisons

Reference: published **ic-stable-structures 0.7.2**, not older Storage or the
optional ICSS Git control. Pending local improvements follow released Storage
0.68.0. No release, sibling adoption or image reset is performed by this study.

## Step 1: Vector final-element shortcut

`last()` reads the final remaining live slot directly. Clip the original iterator
range to the current length, preserving forward-iteration results after shrinking,
appending and front/back consumption. No layout or public type changes. Like
`nth()`, this skips decoding/validation of discarded records. Seven focused
native tests pass, including 240 consumption/length combinations and one returned
record's IO/decode at populations 1/256/2,048. Strict core Clippy passes.

Measured instructions, 256 records, one operation per call, uniform access:

| Value / operation | Storage 0.68.0 | Storage pending | ICSS 0.7.2 |
| --- | ---: | ---: | ---: |
| u64 Get | 81,018 | 81,036 | 81,144 |
| u64 Last | 192,808 | 81,162 | 181,576 |
| fixed 32-byte Get | 81,137 | 81,173 | 81,194 |
| fixed 32-byte Last | 198,522 | 81,236 | 184,189 |
| variable 31-byte Get | — | 161,508 | 161,215 |
| variable 31-byte Last | — | 162,041 | 20,755,454 |
| variable 4 KiB Get | — | 169,654 | 165,274 |
| variable 4 KiB Last | — | 174,333 | 22,866,431 |

The variable type declares a 1 MiB maximum: slot spacing explains why scanning
256 tiny values is much more expensive than the fixed-width scan. The shortcut
does not change that capacity cost: 4,084 Storage pages versus ICSS's 4,089.
The 128–131× variable-value Last ratio is operation-specific, not a general
Vector/endpoint speedup. Both batches 1/32 and every configured Vector operation
were checked against the independent model and final contents. One seed/round;
these are bounded deterministic comparisons, not statistical fleet estimates.

Local raw evidence under `target/evidence/storage-comparison/`: baseline
`attempt.zSR8jI`, fixed-width after `attempt.ExqXui`, variable-width after
`attempt.fYBYaT`. The initial `attempt.5h5Mtp` refused an invalid shared payload
selection before measurement and is not a result. Each completed attempt retains
source/module hashes, source archive, configuration, raw calls and full tables.
Native evidence lives under `target/evidence/storage-068-followup/`.

## Step 2: Map/region attribution and one shared header read

The native `collection-io` diagnostic checks 160 matched scenarios across three
stacks (480 observations): Storage; ICSS over governed Storage backing; ICSS over
its MemoryManager. Populations 256/4,096, fixed 8-byte and variable 32/4,096-byte
values, uniform/hot-16 keys, batches 1/32, Get/Replace and variable growth/read.
Every returned value and final record matches an independent model. Matching
codecs use the same byte order and bounds. This is IO attribution, not timing.

Before the change, Storage's 80 Map observations have identical backing/codec
counts to governed ICSS. Regions read an extra 12-byte admission header per
operation. Admission now shares the engine's root-header read: no duplicate
framing policy, no cached roots, no layout or public error change. All 160 pairs
then have identical backing IO, codec counts and stable-range counts to governed
ICSS. Native physical stable-range sets are **not** heap fault sets or IC fees.
ICSS's own manager may split IO differently; it also forwards some empty writes.

Installed instructions, 256 records, uniform batches of 32; upper medians across
three seeds and three stateful rounds, including intervening mutation phases:

| Operation | Storage before | Storage after | ICSS 0.7.2 |
| --- | ---: | ---: | ---: |
| Map Get | 725,632 | 725,632 | 534,896 |
| Map Replace | 1,435,027 | 1,435,027 | 1,096,733 |
| Region Get | 1,091,570 | 1,077,714 | 562,096 |
| Region Replace | 2,050,724 | 2,036,676 | 1,116,377 |

The eliminated read saves 0.7–1.3% in these region cells. The larger instruction
gap remains despite matching native IO/codec counts; this does not identify how
much belongs to governance, bounds/dispatch, heap placement or metering. Removing
those checks or restoring a cache is not justified by this experiment.
ICSS has no region API: its comparison reloads a map for each operation.

Twenty selected adapter tests pass, including empty/foreign framing admission,
zero writes on refusal, interleaving and map histories. Six independent ICSS
same-image checks pass, plus strict core/diagnostic Clippy. Installed evidence:
`attempt.DFJBmg` before and `attempt.J3enEn` after, 3,888 calls each. Native reports:
`collection-io-final.jsonl` before and `collection-io-after.jsonl` after beneath
the native evidence directory. Failed setup attempts are not counted.

## Step 3: population, churn, growth and bucket matrix

The `depth` preset passed 35,424 calls: 256/4,096 populations, all five collection
forms, shuffled insertion, uniform/hot-16 access, batches 1/32, three independent
seeds and three stateful rounds, with ICSS buckets 1/8/128. Normal operation
histories include insertion/removal/reinsertion, growth/shrink and final checks.
Cell holds one value in each population context. No production sizing knobs change.

Selected single uniform Gets in the 4,096 context; instructions use eight-page
ICSS buckets. Capacities are 64 KiB pages including infrastructure:

| Collection | Storage instructions | ICSS instructions | Storage pages | ICSS pages: 1 / 8 / 128 |
| --- | ---: | ---: | ---: | --- |
| u64 Map | 197,858 | 184,815 | 6 | 4 / 9 / 129 |
| 32-byte Map | 273,534 | 262,179 | 12 | 10 / 17 / 129 |
| u64 Set | 190,004 | 181,260 | 5 | 3 / 9 / 129 |
| u64 Vector | 81,036 | 81,144 | 4 | 2 / 9 / 129 |
| fixed Cell (one value) | 465 | 493 | 4 | 2 / 9 / 129 |
| u64 map region | 215,672 | 185,599 | 6 | 4 / 9 / 129 |

The selected eight-/128-page read instruction counts are identical; one-page
counts differ slightly. Small ICSS buckets can reserve less than governed Storage;
the default 128-page reservation is not a requirement of the tree algorithm.
At 4,096 u64 records, single Vector Last costs 81,162 versus 1,690,696; a batch
of 32 costs 105,912 versus 51,608,597. This remains a Last-specific shortcut.

Another 1,152 calls repeat same-key growth six times, three seeds, both patterns,
both batch sizes, eight-page ICSS buckets, 4,096 records. Uniform batch-32 Grow
costs Storage/ICSS 3,041,898/2,587,570 for 33→132 bytes and
3,287,729/2,839,354 for 80→320 bytes. Storage is still 16–18% slower there.
Selected first/final growth rounds are retained rather than hiding them in medians.

Evidence: `attempt.rS0esw` / `j0xUKb` / `VbBGQK` for buckets 1/8/128;
`attempt.Q8ubIK` for growth. All 44,664 calls across the three steps and their
final states passed. Benchmark library/driver tests pass (27 + 16), plus a new
singleton-context aggregation regression and strict Clippy. After timed runs,
only report aggregation gained `population_context_rows`; its focused test
passes. Selected data derives that context from the original subreports.

[Instruction matrix](../../measurements/storage-068-followup-instructions.csv)
retains 596 selected rows: Get/Replace/Set/Insert/Last/Grow, all growth phases,
plus first/final Grow rounds. `all` means upper median across samples/rounds.
[IO matrix](../../measurements/storage-068-followup-io.csv) retains 120 post-change
uniform batch-32 observations. Complete operations/hot IO remain in raw evidence.

## Harness footprint and evidence scope

Uncompressed Wasm bytes; code size is section 10's decoded payload length:

| Module | Total bytes (delta) | Code bytes (delta) |
| --- | ---: | ---: |
| Storage before | 13,768,991 | 11,576,999 |
| Storage step 1 | 13,775,933 (+6,942) | 11,583,941 (+6,942) |
| Storage step 2 | 13,685,168 (−90,765) | 11,493,756 (−90,185) |
| ICSS, unchanged throughout | 10,888,984 | 9,281,325 |

These are the entire 73-type comparison canisters, not production consumer sizes.
The depth driver reuses step 2's identical Wasm modules. ICSS module SHA-256 is
`4fd4654795eadd2996b0f5a724b205061dda864238522f41d2c38821fe34400b`;
final Storage is `163169f9d71a996d4a068fdaf1c246eb00bf40ef2ad7d9a1a4d880b1c92a2f62`.

All installed runs use Rust 1.99.1, PocketIC 16.1.0, identical comparison profiles
and an 8 MiB synthetic harness stack. ICSS uses eight-page buckets unless stated.
Setup, trace creation, validation and response encoding are outside counters;
collection/backing/codec costs remain inside. Stable capacity includes each
backend's infrastructure. Target evidence remains removable by `cargo clean`.
