# IcyDB-driven B-tree boundary

Source inspection: IcyDB commit `e104fe1c9f5921336f0e5b292acbda99f8311476`,
with its current `icydb-core/src/db` store wrappers as the consumer evidence.
IcyDB itself is not modified by this implementation.

| Observed consumer | Storage requirement | IC Storage surface |
| --- | --- | --- |
| `data/store.rs`, canonical folds | Insert/replace/remove, owned row reads, length | Fallible insert/remove/get, persisted len |
| `data/row.rs`, `codec/mod.rs` | Opaque rows up to 4 MiB | Separate bounded value chains |
| `index/store.rs`, `key_taxonomy.rs` | Component-aware index order, not framed-byte order | Caller-owned Comparator and persistent identity |
| `index/entry.rs` | One-byte presence witnesses | Inline small values |
| `data/store.rs`, `index/store.rs`, `schema/store.rs` | Forward/reverse ranges, key-only preflight, early stop | Lazy double-ended Range and Entry |
| `ordered_overlay.rs` | Ordered canonical/live merge | Lazy entries; consumer adapts fallible traversal |
| `journal/store.rs` | Sequence/chunk keys, 64 KiB chunks | Byte keys and variable-sized values |
| `schema/store.rs`, `integrity/progress_store.rs` | Durable control records and membership | Opaque values and contains_key |
| `ic-memory::RuntimeMemory` | Existing allocation owner | Same Memory trait; no new memory manager |

IcyDB's index comparator compares decoded kind, index identity, component count,
components and primary-key suffix. Comparing its length-prefixed byte envelopes
lexicographically would change query order. The comparator must be implemented
and qualified by IcyDB, where that codec is owned. The test comparator deliberately
uses length-framed keys whose semantic order differs from wire order; it is a
boundary proof, not an implementation of IcyDB's codec.

IcyDB owns journal/live-overlay publication, fold watermarks, schema authority,
exact entity/index cardinalities, continuations and multi-store recovery. A tree
must not independently take those responsibilities or assume all stores share
one physical commit. This implementation stages a single mutation, relies on
IC message rollback for traps during writes and does not add a second journal.

The format uses 4096-byte pages, a checksummed header, 31-entry tree nodes,
32-byte inline keys and 16-byte inline values. Overflow keys and values have
separate chains. Fixed node fanout bounds node decoding independently of a
4 MiB row. Mutations rewrite touched nodes and changed payloads; deletes borrow
or merge and recycle pages. Range cursors retain bounded search paths and do
not load overflow values until requested. Replacement preserves tree topology
and rewrites the existing value chain through mutation staging. It frees surplus
pages when shrinking and allocates only extra pages when growing. Returned
capacity errors preserve all existing bytes, including reused payload pages.

The format has its own magic and never claims compatibility with DFINITY B-tree
pages. IcyDB adoption requires its own comparator/codec adapter, propagation of
fallible operations through overlay and recovery boundaries, a retained-data
transition decision, native/PocketIC qualification and workload benchmarks.
These are integration acceptance requirements, not claims established by this
repository's native model tests or Wasm compilation.

The accepted transition is a hard cut: IcyDB will use this tree without a legacy
reader or dual-format path. This does not authorize discarding retained data;
the application must still choose and qualify its rebuild/import or explicit
reset before switching memory regions. The upstream Memory interface remains
the shared substrate after the cut.

## Native memory-traffic qualification

Run the opt-in synthetic workload with:

```sh
cargo test --locked --test btree workload_memory_traffic -- --ignored --nocapture
```

It counts bytes and calls to Memory read/write methods and reports reserved
capacity after each operation. Setup and structural audits are outside the
measured operations. Fixtures use eight-byte ordered keys: 2,048 one-byte index
markers, 64 journal-sized 64 KiB values, and four 4 MiB rows. Scans stop after
16 entries (or exhaustion) and inspect keys/value lengths without loading values.

On Linux with Rust 1.99, comparing released 0.1.1 (`19d53cc`) with the pending
0.1.2 implementation and the same fixture:

| Single same-size replacement | 0.1.1 read bytes | 0.1.2 read bytes | Write bytes, both |
| --- | ---: | ---: | ---: |
| Index marker | 24,576 | 12,288 | 8,192 |
| Journal-sized value | 155,648 | 77,824 | 77,824 |
| Row | 8,454,144 | 4,227,072 | 4,231,168 |

The replacement now reuses its search result and assembles the returned old
value while reading its chain once. Measured get, key-scan, removal, write-byte
and reserved-capacity results are unchanged. These measurements count interface
traffic, not heap allocations, wall-clock speed, IC instructions or cycles.

## Direct upstream qualification

`tests/upstream.rs` compares our tree, ic-stable-structures 0.7.2 and an independent
heap map. Both persisted trees use separate memory regions. Keys have a length
prefix whose byte order differs from semantic order: upstream uses a typed `Ord`,
ours a byte comparator, and the heap model stores decoded keys. Checks cover old
values returned by mutations, membership, length, full contents, mixed-direction
ranges, valid endpoint combinations, overflow boundaries, maximum payloads,
reopening, draining and refilling. Real upstream bytes must be rejected without
modification. Invalid-range, malformed-key, corruption and capacity contracts
are qualified separately by our own tests; upstream's panic/initialization
behavior is not treated as our required contract.

```sh
cargo test --locked --test upstream
cargo test --release --locked --test upstream extended_seeded_operations_match_upstream_and_model -- --ignored --nocapture
cargo test --release --locked --test btree workload_memory_traffic -- --ignored --nocapture
cargo test --release --locked --test btree bounded_upstream_index_memory_traffic -- --ignored --nocapture
```

The regular campaign runs four fixed seeds with 2,000 operations each. The
explicit extended campaign runs seeds 0 through 31 with 10,000 operations each;
each seed also prefills, drains and refills 768 keys. Both campaigns passed on
Linux/Rust 1.99 for the pending 0.1.2 source; all four regular upstream tests also
passed on Rust 1.85. This is deterministic coverage, not exhaustive proof.

The traffic fixture now runs the same setup and operation order against both
trees, reporting each operation separately. The baseline upstream fixture uses
unbounded `Vec<u8>` keys and values. A second upstream fixture uses fixed eight-byte
keys and one-byte values, demonstrating the effect of its bounded layout. Neither
fixture uses IcyDB's actual codecs or runtime memory manager.

Representative same-size replacement results (read bytes / calls):

| Workload | Our tree | Upstream unbounded | Upstream bounded |
| --- | ---: | ---: | ---: |
| Index marker | 12,288 / 3 | 601 / 85 | 509 / 62 |
| 64 KiB value | 77,824 / 19 | 337,969 / 1,280 | Not measured |
| 4 MiB row | 4,227,072 / 1,032 | 16,959,466 / 33,140 | Not measured |

The marker replacement writes 8,192 bytes here versus 108 unbounded or 88 bounded
upstream bytes. Reserved index capacity is 589,824 bytes here versus 458,752
unbounded or 131,072 bounded upstream bytes. Our full-page layout has a clear
small-entry byte/storage cost despite fewer interface calls. Larger-payload
replacements and key-only scans reduce traffic in the unbounded comparison.
This does not establish an overall performance winner: bounded large-value
layouts, actual IcyDB schemas, heap usage, IC instructions and rollback/upgrade
behavior still require qualification before adoption.

## IcyDB bound-matched profiles (0.1.3)

The next fixture copies only the `Storable::BOUND` declarations from committed
IcyDB revision `e104fe1c9f5921336f0e5b292acbda99f8311476`. It does not depend on
the sibling checkout, import its dirty working tree, or reproduce its codecs.
Opaque synthetic keys preserve byte lengths and ordering. Both trees receive
the same bytes, are reopened before measurement, and are compared after mutations
and another reopen. This isolates layout decisions from application encoding.

| Profile | Key bound | Value bound | Source within `icydb-core/src` |
| --- | --- | --- | --- |
| Index | 16,668 bytes, variable | 1 byte, variable | `db/index/key/codec/{bounds,envelope}.rs`, `db/index/entry.rs` |
| Journal | 12 bytes, fixed | 65,536 bytes, variable | `db/journal/store.rs` |
| Row | 262 bytes, variable | Unbounded (application limit 4 MiB) | `db/data/{key,row}.rs` |

The composite primary-key maximum is `2 + 4 * (1 + 62) = 254` bytes, using
`db/key_taxonomy.rs` and `types/account.rs`. Data keys add an eight-byte entity
tag. Index keys add the 20-byte prefix, four `(2 + 4096)` component envelopes,
and a `(2 + 254)` primary-key envelope, giving 16,668 bytes. Row size admission
does not make its upstream `Storable` declaration bounded.

```sh
cargo test --locked --test btree icydb_bound_profiles_reopen_and_mutate
cargo test --release --locked --test btree icydb_bound_profiles_memory_traffic -- --ignored --nocapture
```

Linux/Rust 1.99 results for the pending 0.1.3 fixture against upstream 0.7.2:

| Profile (entries; actual key/value bytes) | Capacity ours / upstream | Replacement read bytes ours / upstream |
| --- | ---: | ---: |
| Index (2,048; 24 / 1) | 589,824 / 56,098,816 | 12,288 / 729 |
| Index (2,048; 64 / 1) | 8,978,432 / 56,098,816 | 57,344 / 1,249 |
| Index (512; 1,024 / 1) | 2,293,760 / 14,090,240 | 45,056 / 13,649 |
| Journal (64; 12 / 4,096) | 589,824 / 5,963,776 | 16,384 / 20,814 |
| Journal (64; 12 / 65,536) | 4,521,984 / 7,077,888 | 77,824 / 328,025 |
| Row (64; 25 / 4,096) | 589,824 / 327,680 | 16,384 / 21,518 |
| Row (4; 25 / 4,194,304) | 16,908,288 / 17,235,968 | 4,227,072 / 16,959,545 |

Capacity is measured after replacement, before deletion. Upstream chooses its
node allocation size from both declared bounds when both are bounded; these
wide index-key bounds therefore change capacity dramatically from the earlier
eight-byte fixed-key fixture. Our 64-byte keys exceed the 32-byte inline limit
and each requires an overflow page, explaining our own sharp capacity increase.
The earlier compact-bound result remains valid for that fixture, not for IcyDB's
declared index layout.

Read/write byte traffic still favors upstream for tiny index entries; our tree
uses fewer calls. For bounded journals, replacements use less traffic here, but
upstream key-only scans use fewer bytes (535 versus 8,192 for 64 KiB chunks).
Large-row replacement benefits persist because IcyDB rows are unbounded upstream.
These results do not justify an overall speed claim or a format redesign yet:
measure actual canister instructions, real encoded-key distributions, heap usage,
and application rollback/upgrade behavior before deciding adoption or tuning.

### Churn and overflow reclamation

The same profile command now continues with restore, shrink-all to at most 16
bytes, regrow-all to original values, delete even-numbered keys and refill those
keys in reverse order. One-byte index values remain one byte during the shrink
and grow phases. Bulk counters are phase totals, not per-operation averages.
Each phase audits our tree and reopens both trees outside the measured counters.
Returned old values are checked throughout; final contents and counts must agree.

All seven profiles retain our original reserved capacity through churn. This is
evidence for these fixtures, not a guarantee that arbitrary reorderings never
need more node pages. Upstream also reuses capacity in most fixtures; its 64 KiB
journal profile grows from 7,077,888 to 7,602,176 bytes during the first deletion.

The deletion measurements exposed a redundant read: chain traversal read each
overflow page, then freeing it read it again. Returning the already-validated
prior page from reclamation eliminates that third payload pass (returning the
old value still needs its own read). All updates remain staged; a chain/type
error after staging discards the transaction before any writes are published.

| Single removal | Prior read bytes | Revised read bytes | Write bytes, both |
| --- | ---: | ---: | ---: |
| 64 KiB journal value | 233,472 | 163,840 | 86,016 |
| 4 MiB row | 12,677,120 | 8,454,144 | 4,231,168 |

Native byte counts cannot account for checksum CPU cost, allocation or IC system
call pricing. IC Testkit already owns canister counter emission and PocketIC
lifecycle helpers; a separate benchmark canister/host runner is still required
here to obtain instruction and heap evidence without raising the library's MSRV.
The following section records the initial implementation of that harness.

## Canister instruction baseline

`ic-storage-bench` is a non-published workspace package with a release Wasm
canister and native runner. Its tooling requires Rust 1.99; `ic-storage` retains
Rust 1.85 and its original dependency graph. IC Testkit owns counter capture,
Candid calls and managed server lifetime. IC Host FS owns bounded artifact reads
and hashes. Each run uses a fresh application-subnet PocketIC instance and one
canister owning its complete fresh stable-memory region; no allocation manager
or IcyDB installation is involved.

```sh
make bench-run POCKET_IC_BIN=/absolute/path/to/admitted/pocket-ic
```

The explicit server must satisfy Testkit's 16.x protocol admission. The runner
does not install/download it. It emits CSV to stdout and artifact hashes/errors
to stderr. Preserve both outputs in a new attempt directory. The release Wasm
is built by `bench-build`; native runner optimization does not affect canister
instruction counters. Setup uses bounded batches in separate messages. Two fresh
runs per engine/profile execute get, same-size replacement, a key-only scan of
up to 16 entries, then removal. Engine 0 is ours; engine 1 is upstream 0.7.2.

The profiles match the declared bounds above: 2,048 index entries with 24-byte
keys/one-byte values; 64 journal entries with 12-byte keys/64 KiB values; four
rows with 25-byte keys/4 MiB values. Encoded bytes are synthetic and ordered
lexicographically, not IcyDB's real codecs or comparator. Input construction,
seeding, result validation and Candid reply encoding are outside the measured
interval. Dispatch, storage operations and result materialization are inside.
There is small counter-read overhead; counts are not adjusted by a blank probe.
Heap bytes are total Wasm memory capacity after the operation, not live heap
allocations, peak usage, or a per-operation allocation delta. Stable bytes are
reserved stable capacity. Results are asserted after measurement in the canister.

Observed on Linux, Rust 1.99 release Wasm, IC Testkit 0.48.0, IC CDK 0.20.3,
PocketIC client 16.1.0 and server 16.0.0. Both fresh runs produced identical
instruction counts for each engine/profile/operation:

| Profile | Operation | Our instructions | Upstream instructions |
| --- | --- | ---: | ---: |
| Index | Get | 180,243 | 350,507 |
| Index | Replace | 625,276 | 441,410 |
| Index | Key scan | 244,666 | 576,678 |
| Index | Remove | 2,330,403 | 1,703,767 |
| Journal | Get | 1,236,339 | 564,457 |
| Journal | Replace | 4,003,353 | 2,522,514 |
| Journal | Key scan | 170,608 | 388,837 |
| Journal | Remove | 5,742,443 | 10,818,500 |
| Row | Get | 68,748,732 | 37,953,614 |
| Row | Replace | 215,655,911 | 124,318,147 |
| Row | Key scan | 74,710 | 23,021,659 |
| Row | Remove | 265,860,686 | 120,852,456 |

The small-entry and large-record tradeoffs are real: fewer/larger stable calls
and lower byte traffic do not account for our checksums, page decoding, staging
and allocation work. These results locate operations needing investigation;
they do not isolate which of those costs dominates. Do not remove integrity
checks or change persisted layout based on traffic counts alone. These runs do
not prove application recovery, upgrade safety, actual IcyDB performance, or
production-subnet cycle costs; macOS simulator execution is also unqualified.

Evidence from this working tree is retained under `target/canister-bench.E706g7`
(`results.csv`, `run.log`). Wasm SHA-256:
`b3e5b6f4b949d78553dd3be420c9dcaca4fba218d65e08d9baa00eddecd457f8`.
Server SHA-256:
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
The initial sandbox listener failure is retained in `target/canister-bench.RV48RT`;
the initial reply-decoder mismatch is retained in `target/canister-bench.rECDFS`.
The successful run uses the corrected top-level tuple reply decoder.

### Checksum and staging investigation

Two sequential changes isolate useful implementation costs without changing
the format or skipping validation. First, iterate the checksum in eight-byte
chunks so the compiler can reduce loop control while retaining exactly the
same FNV-1a byte sequence. Second, store staged pages as boxed buffers: the
heap B-tree then moves pointers during balancing instead of 4 KiB arrays.
This adds individual page allocations but reduces staging movement and capacity
in the large-row fixture. Reads still copy out a validated page; publication
still reserves capacity before writing staged pages and the header.

The same runner, server and input profiles produced the following counters.
Each column represents two fresh runs with identical results within that build.
The baseline above is preserved rather than relabeled as current evidence.

| Profile | Operation | Baseline | Chunked checksum | Plus boxed staging |
| --- | --- | ---: | ---: | ---: |
| Index | Get | 180,243 | 163,380 | 163,460 |
| Index | Replace | 625,276 | 597,171 | 589,351 |
| Index | Key scan | 244,666 | 227,803 | 228,964 |
| Index | Remove | 2,330,403 | 2,201,120 | 2,127,611 |
| Journal | Get | 1,236,339 | 1,129,540 | 1,129,540 |
| Journal | Replace | 4,003,353 | 3,789,755 | 3,063,688 |
| Journal | Key scan | 170,608 | 159,366 | 158,985 |
| Journal | Remove | 5,742,443 | 5,393,941 | 4,602,388 |
| Row | Get | 68,748,732 | 62,947,860 | 62,947,860 |
| Row | Replace | 215,655,911 | 204,048,546 | 146,834,753 |
| Row | Key scan | 74,710 | 69,089 | 69,089 |
| Row | Remove | 265,860,686 | 248,452,449 | 195,445,266 |

Combined row replacement improves by 31.9% and removal by 26.5%. Row heap
capacity after replacement drops from 17,367,040 to 13,893,632 bytes; stable
capacity remains 16,908,288 bytes. This is total capacity after prior setup and
operations, not a live allocation measurement. Current upstream row replacement
is 124,318,141 instructions and removal 120,852,456: our mutations still cost
more. Small upstream instruction shifts between builds (at most 74 instructions)
also occur despite unchanged upstream source; comparisons are measurements of
whole compiled artifacts, not perfect per-function attribution. Likewise the
small index read/scan shifts after boxing must not be attributed to page movement.

The checksum-only result shows loop overhead matters, but does not establish
the total cost of hashing. Boxing strongly reduces mutation costs in this
fixture; it does not isolate allocation versus copying versus index traversal.
No integrity check, staged-write guarantee or stored byte was removed. Tests
compare 66 zero, all-one and seeded page patterns with the original algorithm
and reject a bit flip at each of all 4,096 page positions, including checksum
bytes. Regular storage failure/reopen tests and upstream comparisons also pass.

Intermediate evidence: `target/checksum-bench.v3qZaS/{results.csv,run.log}`;
Wasm SHA-256
`cb0b2e2dceec2185e2dcfd89187c949ce8b9ec82a3efdb681a2ef4b193705107`.
Combined evidence: `target/boxed-pages-bench.M1NRYV`, including CSV, log and
the measured Wasm; SHA-256
`483dd63d0067e354c209830914a2e81ecc2cf26425a3dd1fc83692adf704f05b`.
The server identity and qualification limitations are unchanged from the baseline.

### Expanded fixtures and free-page construction

The current runner shares its [fixture descriptors](../../crates/ic-storage-bench/src/fixtures.rs)
between host and canister. There are 15 profiles: short index keys at 128,
2,048 and 4,096 entries; 256-byte and maximum 16,668-byte index keys; the journal
profile; row values of 16, 17, 4,072, 4,073, 16,384, 65,536, 1,048,576 and
4,194,304 bytes; and 1,024 rows with maximum 262-byte keys and 4,073-byte values.
The descriptors specify exact population and bounds; names do not assert
identical tree height in both implementations. Upstream bound metadata remains
the same as the earlier IcyDB-matched index, journal and row layouts.

`make bench-run POCKET_IC_BIN=/absolute/path/to/admitted/pocket-ic` runs both
engines twice from fresh installations. Each profile executes, in order: get,
missing get, same-size replacement, key scan, value scan, shrink to one byte,
regrow, removal, refill and insertion of a new greatest key. Scans materialize
up to 16 entries. The one-byte index profiles intentionally make shrink/regrow
same-size replacements. Setup uses ordered keys and repeated-byte values;
these are not random insertion distributions or real IcyDB codecs. Replacement
writes the same bytes. Mutations assert both returned and stored values outside
the measured interval. The workload is sequential, not an isolated fresh tree
for every operation; earlier operations and validation affect allocator history.

Value scans now extend the output by whole slices for both engines. The first
expanded attempt used byte-by-byte iterator flattening, which substantially
inflated result-materialization cost: our four-row 4 MiB scan fell from
1,781,752,911 to 322,134,408 instructions after this harness correction, without
changing the storage code. This is not a B-tree optimization. Both figures
include copying into an output buffer; neither is a zero-copy cursor benchmark.

The production change is limited to constructing free pages. Their first
16 bytes contain the tag and next pointer, followed by known-zero padding.
FNV-1a on a zero byte is multiplication by its prime modulo 2^64, so a constant
power covers the entire zero tail. Only construction uses this shortcut; reads
still checksum all bytes. Tests compare full page bytes against ordinary sealing
for 130 successor patterns and reject a flipped bit at every page position.

With the corrected harness, 4 MiB shrink drops from 196,375,461 to 156,797,740
instructions and removal from 195,445,228 to 155,867,505 (about 20.2% each).
Replacement remains 146,834,731. The corresponding upstream counts are
120,840,616, 120,840,013 and 124,318,115. The improvement does not close the gap.

The following ratios are **our instructions / upstream instructions** after
the free-page change; below 1 is cheaper, and values are rounded to two decimals.
These are per-fixture comparisons, not a workload-weighted overall score.

| Profile | Get | Replace | Remove | Value scan |
| --- | ---: | ---: | ---: | ---: |
| index_128 | 0.41 | 1.07 | 0.74 | 0.31 |
| index_2048 | 0.47 | 1.34 | 1.20 | 0.36 |
| index_4096 | 0.39 | 1.13 | 1.00 | 0.32 |
| index_long | 1.77 | 2.46 | 1.99 | 1.87 |
| index_max | 1.21 | 1.23 | 1.48 | 2.51 |
| journal | 2.00 | 1.21 | 0.37 | 1.84 |
| row_inline | 1.07 | 1.96 | 3.33 | 1.00 |
| row_overflow | 1.54 | 4.18 | 6.07 | 6.02 |
| row_page | 0.45 | 1.64 | 0.86 | 1.72 |
| row_two_pages | 0.58 | 1.85 | 0.92 | 2.77 |
| row_16k | 0.46 | 0.91 | 0.34 | 1.70 |
| row_64k | 0.58 | 0.80 | 0.30 | 1.73 |
| row_1m | 1.65 | 1.19 | 1.30 | 1.60 |
| row_4m | 1.66 | 1.18 | 1.29 | 1.60 |
| row_deep | 0.67 | 1.09 | 0.64 | 2.29 |

The 17-byte fixture exposes the current 16-byte inline cutoff: just one more
byte requires a separate 4 KiB overflow page. Long keys also require overflow
reads during comparisons. These are concrete weaknesses of this layout; the
matrix does not isolate all allocation, hashing and traversal costs. Mid-size
row wins and short-key scan wins do not establish an overall advantage, and
value scans show a different result from key-only scans. No format change or
recovery qualification is implied by this investigation.

Each attempt retained CSV, log and measured Wasm, with 600 rows and identical
counters/result lengths between its two fresh runs:

- Initial expanded harness: `target/expanded-bench.cnGYm0`, Wasm SHA-256
  `002bc4acc26b0e2d2d35918c16cc4a42583ede61d14e5bc54f72e3eef14ed986`.
- Corrected bulk-scan baseline: `target/expanded-bulk-bench.O8IJF5`, Wasm SHA-256
  `92aff0536b557689c5fa8dfcc3da3891d6c644333fb145c1bd1dadeaf8ecfdf7`.
- Free-page optimization: `target/free-page-bench.qf9NDM`, Wasm SHA-256
  `28501b7a20a1bfbe78fbbfa7cce2ff4b899145cce0b33dc380a4346fe0e085a0`.

Compiler, server and dependency selections match the earlier baseline.
The measured build precedes lint-only literal formatting and an equivalent
nonempty-name assertion in canister initialization. Native regular and extended
differential checks, Rust 1.85 storage tests and native/Wasm Clippy pass; macOS,
actual IcyDB codec mixes, random insertion profiles, bounded/reverse canister
scans and rollback/upgrade recovery remain unqualified.

### Zero-suffix checksum qualification after 0.1.3

The checksum now scans backward over complete eight-byte zero chunks before
hashing the remaining prefix. Each zero chunk is inspected, then represented
by multiplication by the FNV prime raised to eight; the accumulated suffix
factor is applied after the prefix. This is exactly the original FNV-1a result,
including nonzero corruptions in padding. It does not trust a page tag or assume
that reserved bytes are zero. Full payloads incur only the failed suffix test
and final multiplication. The free-page construction shortcut remains separate:
it knows the newly constructed padding, while this read/seal path verifies it.

A new test compares with the original algorithm at all 4,088 positions of the
last nonzero covered byte. Existing seeded patterns and corruption at every
page position also pass, including stored checksum bytes. No persisted byte,
inline threshold, API, read-traffic or mutation publication contract changes.

To distinguish padding gains from synthetic-key bias, every fixture now runs
with two key suffix patterns: zeros, and `0xa5` bytes (`_dense` in CSV names).
Both retain the big-endian counter in the first eight bytes and the same sizes,
bounds, insertion order and nonzero value bytes. The current benchmark's init
arguments are `(engine, profile, dense_keys)`; the runner owns this non-published
contract and the `make bench-run` command is unchanged. These patterns exercise
zero-rich versus nonzero-rich bytes, not real codecs or random key order.

Each controlled build produces 1,200 measurements: 15 fixtures, two patterns,
ten operations, two engines and two fresh runs. Baseline and optimized builds
use this same expanded harness and locked dependency graph. Across all 300 of
our operation/profile/pattern combinations, instructions fall by 0.67–53.13%.
All counters and result lengths repeat exactly within each build. This is a
fixture result, not a general performance guarantee or weighted workload score.

Selected **nonzero-key** results, in instructions:

| Fixture | Operation | Before | After | Upstream after |
| --- | --- | ---: | ---: | ---: |
| 17-byte row | Get | 154,593 | 96,514 | 100,118 |
| 17-byte row | Replace | 796,659 | 663,868 | 190,439 |
| 17-byte row | Value scan | 901,701 | 430,162 | 149,871 |
| 17-byte row | Remove | 1,672,655 | 1,396,344 | 274,168 |
| 4 MiB row | Get | 62,948,166 | 62,386,894 | 37,954,113 |
| 4 MiB row | Replace | 146,834,890 | 145,687,062 | 124,318,245 |
| 4 MiB row | Value scan | 322,135,008 | 319,966,993 | 200,787,801 |
| 4 MiB row | Remove | 155,867,785 | 154,691,671 | 120,839,995 |

Nonzero 256-byte index-key lookup falls from 494,150 to 256,033 instructions
(48.2%), while maximum 16,668-byte index-key lookup falls from 2,047,774 to
1,854,746 (9.4%). Zero-filled maximum keys showed a much larger 42% improvement:
reporting only that pattern would overstate the general long-key benefit.
Small overflow reads are now competitive in this fixture, but mutation and
value-scan costs still expose the extra page required above the 16-byte inline
limit. Large nonzero payload improvements remain below 1% for these operations.

Controlled baseline: `target/key-pattern-baseline.SaviDD`, Wasm SHA-256
`ce98435a3265de89bc5507eda89102e9a725dde0ed5e2578c8238254c77dcca9`.
Optimized: `target/key-pattern-optimized.eSra9b`, Wasm SHA-256
`7ba9ed4061802203e71874b9d0c076fec6e1886d32a760410cda8fea0664007f`.
Both directories retain the measured Wasm, CSV and log. Source starts from
release `5906b48` with the incoming IC Host FS 0.22/locked 0.22.1 and IC Metrics
0.11.5 changes preserved; the baseline adds the key-pattern harness but retains
the original checksum. Compiler, PocketIC and other selections match the earlier
setup. No dependency change is attributed to this optimization.

The preliminary zero-key-only comparison is also retained:
`target/zero-chunk-baseline.hwlJKj` (Wasm SHA-256
`050d8d887688fad013a1410dc02a1f1892b397a81a9c87f1cb71c852ed554325`)
and `target/zero-suffix-bench.re7TwI` (Wasm SHA-256
`a97c36881c80cbf074c75bac51bddbf192581bfc56ae07f52947f37bc73c8ad0`).
Native differential, failure/reopen and Rust 1.85 checks pass; the prior macOS,
real-codec and canister recovery qualification limits still apply.

### Single-pass value reclamation

Deletion previously called `get` to materialize the returned value before
starting its transaction, then traversed the value pages again when freeing
them. It now checks key existence without loading the value and collects the
previous bytes from the validated pages returned during reclamation. Inline
values are copied from the removed record. Key pages are reclaimed without
collecting their bytes. Node changes, free-list changes and the returned value
remain staged until validation and capacity checks complete; no application
error after partial staging publishes any page.

The read-budget regression now allows only one payload pass plus bounded
node overhead; it failed before this change and passes afterward. A new test
corrupts the tail of a multi-page value, checks `Error::Corrupt`, zero backing
writes, unchanged bytes and length, then repairs only that injected byte and
successfully removes/reopens the tree. Existing split/merge, replacement-record,
custom-order, corruption and differential tests also pass. This remains native
failure evidence, not qualification of canister traps or upgrade recovery.

The unchanged two-pattern canister harness produces these nonzero-key results:

| Removed value | Previous instructions | Single-pass instructions | Upstream |
| --- | ---: | ---: | ---: |
| 17 bytes | 1,396,344 | 1,377,786 | 274,170 |
| 64 KiB | 3,686,780 | 2,920,415 | 13,186,662 |
| 1 MiB | 38,948,493 | 26,995,276 | 30,225,135 |
| 4 MiB | 154,691,671 | 106,900,172 | 120,839,997 |

The 4 MiB fixture improves 30.9% against the prior implementation and is 11.5%
cheaper than upstream. The read/copy needed to return the payload still exists;
only the redundant traversal is removed. Small overflow removal remains costly.
The 16-byte inline fixture increases from 696,469 to 697,290 instructions
(0.12%) with either key pattern; it has no overflow traversal to eliminate.
These are per-operation synthetic results, not an overall workload victory.

All 1,200 measurements repeat exactly across the two fresh runs. The prior
artifact is `target/key-pattern-optimized.eSra9b` above; the new CSV, log and
measured Wasm are retained in `target/single-pass-delete.9v83Hq`, SHA-256
`97c1735b41393640278151ec04ac2c4d7d73164e3eaaa5e1bb99ee3aff7593b5`.
Dependency, compiler, server and fixture selections are unchanged. Four checksum
tests and 19 storage tests pass on Rust 1.99 and 1.85; four upstream tests and
the 320,000-operation extended campaign pass on 1.99. Native/Wasm Clippy passes.
