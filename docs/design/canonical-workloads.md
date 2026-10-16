# Canonical-store workload investigation

The initial investigation starts from IC Storage
`654a2d706982fe7433b7ce432fe1e86592766a4e` (released 0.1.6), with the benchmark
additions in this working tree. It initially left production storage code
unchanged; the compatible optimization below follows that baseline. Format,
dependencies and release metadata remain unchanged throughout.

## IcyDB ownership and copy trace

Read-only source inspection uses committed IcyDB
`996498ad4edb5569964b68e0c0e3f5bb9df7d1c6`. Its sibling checkout contains unrelated
uncommitted codec work; that work is neither included in this evidence nor edited.
These are source observations, not a deployed IcyDB profile:

- [RawRow](https://github.com/dragginzgame/icydb/blob/996498ad4edb5569964b68e0c0e3f5bb9df7d1c6/crates/icydb-core/src/db/data/row.rs#L86)
  borrows in `to_bytes`, moves in `into_bytes`, and retains owned `from_bytes`
  input through `Cow::into_owned`. `RawJournalChunk` does the same. Removing
  `Storable` does not automatically remove a payload clone from these paths.
- [DataStore::read](https://github.com/dragginzgame/icydb/blob/996498ad4edb5569964b68e0c0e3f5bb9df7d1c6/crates/icydb-core/src/db/data/store.rs#L537)
  borrows heap/live-overlay rows and owns canonical stable reads. Tombstones
  avoid canonical reads. Canonical journal folds call stable insert/remove;
  application writes do not all immediately become canonical tree writes.
- The same store's key visitors use lazy range keys without requesting values;
  row visitors request the owned canonical value. Both support reverse order
  and early exit. IcyDB owns merging canonical/live entries and tombstones.
- [RawDataStoreKey](https://github.com/dragginzgame/icydb/blob/996498ad4edb5569964b68e0c0e3f5bb9df7d1c6/crates/icydb-core/src/db/data/key.rs#L338)
  borrows `to_bytes`, retains owned decoding input, but copies in its `Storable`
  `into_bytes`. That is not evidence of a copy on every tree request: upstream
  0.7.2 `btreemap::node::v2` serialization uses `key.to_bytes_checked()`.
  Its insertion consumes **values** with `into_bytes_checked()`. The key helper
  alone therefore does not explain the measured read gap.
- Raw index keys and index witnesses borrow serialization and move their owned
  buffers. Index order is codec-aware, not lexicographic envelope order.
  Journal keys encode their sequence/chunk integers into a fresh 12-byte buffer.
  These codec/comparator costs are not represented by our synthetic byte wrappers.

The existing synthetic `Bytes` wrapper already borrows serialization and moves
owned payload buffers. The new scans copy the requested projection directly into
one output buffer, without the old harness's intermediate per-key `Vec` clone.
This deliberately retains the older matrix unchanged as a separate experiment.

## Reproducible workload

```sh
make bench-build
cargo build -p ic-storage-bench --bin ic-storage-bench --locked
target/debug/ic-storage-bench "$POCKET_IC_BIN" \
  target/wasm32-unknown-unknown/release/ic_storage_bench.wasm workloads
```

Use an explicitly admitted PocketIC 16.x server. Omitting `workloads` still runs
the existing 23-fixture matrix. The new suite uses two profiles, two engines,
four ordered rounds and two fresh repetitions: 256 CSV measurements.

Each profile seeds 512 entries in the permutation `(i * 149 + 37) % 512`.
Keys cycle through 24/32/33/64/128/262 bytes with nonzero suffixes. Row values
cycle through 1/16/17/32/256/1,024/4,072/4,073/8,192/65,536 bytes; payload bytes
vary deterministically. Index values are one-byte witnesses. Upstream bounds
remain 262/unbounded for rows and 16,668/1 for index keys/values. These are
source-informed size and call shapes, **not production frequency estimates or
actual IcyDB encoded records**. In particular, long keys are deliberately common.

Each round performs 32 reads (24 hits/eight misses), 32 changed puts with varying
row sizes, 32 deletes, 32 refills, then four bounded scans: keys/values in each
direction, stopping after 16 entries. Mutation targets follow a deterministic
permutation and shift between rounds. Scans include the lower endpoint and
exclude the upper. Later phases inherit allocation history; results describe
this sequence, not isolated-operation averages or a workload-weighted score.

Inputs and independent heap-model expectations are prepared before timing.
Returned bytes are checked afterward. Every mutation batch also checks all 512
keys and audits our entire tree outside timing. Both engines use the same model
and inputs; repeat/result-length agreement is checked across CSVs. The measured
interval includes tree calls, result allocation and projection copying, but not
fixture generation, model checks, reply encoding or seeding. Heap/stable columns
are total capacities after the call, not allocated-byte deltas or live heap size.
No overlay, actual comparator, canister upgrade or rollback claim follows.

## Results

Instruction totals below sum four rounds from one repetition, in millions.
Both repetitions agree exactly in instructions, capacities and returned lengths.
A ratio below one favors IC Storage. Do not combine these totals into a general
IcyDB speedup: the operation mix and frequencies have not been measured there.

| Profile | Operation | Ours M | Upstream M | Ours/upstream |
| --- | --- | ---: | ---: | ---: |
| Rows | Read mix | 38.327 | 28.922 | 1.33 |
| Rows | Fold put | 75.917 | 74.831 | 1.01 |
| Rows | Fold delete | 106.847 | 86.686 | 1.23 |
| Rows | Refill | 98.608 | 72.443 | 1.36 |
| Rows | Forward keys | 2.381 | 8.432 | 0.28 |
| Rows | Reverse keys | 2.394 | 8.713 | 0.27 |
| Rows | Forward values | 12.297 | 15.043 | 0.82 |
| Rows | Reverse values | 10.004 | 14.003 | 0.71 |
| Index | Read mix | 26.095 | 12.423 | 2.10 |
| Index | Fold put | 35.953 | 25.599 | 1.40 |
| Index | Fold delete | 76.581 | 32.671 | 2.34 |
| Index | Refill | 66.178 | 29.177 | 2.27 |
| Index | Forward keys | 2.293 | 1.781 | 1.29 |
| Index | Reverse keys | 2.145 | 1.869 | 1.15 |
| Index | Forward values | 2.319 | 1.825 | 1.27 |
| Index | Reverse values | 2.173 | 1.905 | 1.14 |

After the final phase, row stable capacity is 7,274,496 bytes versus 4,587,520
upstream; index capacity is 1,507,328 versus 9,830,400. Our broad index bounds
save capacity here despite costing more instructions. Final heap capacity is
6,684,672/6,750,208 bytes for rows and 1,245,184/1,245,184 for index (ours/upstream).

## Isolated small-value prototype

An artifact-only copy under `target/layout-experiment` changes inline values
from 16 to 48 bytes and record stride from 80 to 112, retaining 31-entry nodes
and 32-byte inline keys. Both encode/decode and mutation thresholds change.
Its distinct `ICEXP481` magic prevents confusing experimental and current bytes.
All installations use fresh memory. The exact delta is retained as
`target/inline48.nJNbSC/prototype.patch`; no alternative reader or production
feature flag is introduced.

The 128-entry, 32-byte-row fixture reserves 65,536 bytes instead of 589,824;
its dense-key value scan improves 62.07%. But lookup increases 1.29% and changed
replacement 7.36%. Dense one-byte index lookup increases 21.51%. In the complete
matrix only 28/506 cases improve and 478 regress; upstream counters are unchanged.
Mixed-index operation totals regress 4.62–11.62%. Mixed-row read/put/delete/refill
regress 4.19–7.37%; reverse scans improve, while forward scans regress.

**Rejected for adoption.** More inline space does not by itself make the tree
better: larger fixed records impose costs on unrelated entries. These findings
favor investigating compact variable-sized records or shared small-payload
storage, with explicit allocation/reclamation and corruption proofs, before
committing to another persisted format. The experiment is not a production-
qualified format and does not run the old layout-specific corruption fixtures.

## Large-read cost isolation

A second artifact-only build under `target/read-diagnostic` uses distinct
`ICDIAG01` magic and bypasses only `page::read_into` checksum verification.
Bounds, node decoding and chain checks remain, as does header verification.
The exact delta is in `target/read-diagnostic-run.eTBfty/diagnostic.patch`.
This deliberately weakened build is a cost diagnostic, **not an optimization
candidate**; it must never read application data or replace integrity checking.

With dense keys, a 4 MiB lookup falls from 53,742,872 to 18,919,035 instructions
(-64.80%), versus 37,954,113 upstream. The four-row value scan falls from
270,971,689 to 131,697,434 (-51.40%), versus 200,787,292 upstream. Thus verification
and its associated control flow dominate the measured large-read gap. Eliminating
another `Storable` payload clone cannot explain this result. A faster checksum
would require a deliberate format decision plus corruption qualification; a
cache cannot simply skip validation of bytes read from stable memory.

## Evidence identity and qualification

All runs use Linux, Rust 1.99.0, the unchanged 0.1.6 lockfile, IC Testkit 0.49.0,
IC Host FS 0.23.1, IC Metrics 0.11.6, upstream 0.7.2 and the already admitted
PocketIC server 16.0.0/client 16.1.0. Each attempt retains Wasm, runner, CSVs and
logs. SHA-256 identities:

| Attempt | Wasm SHA-256 | Measurements |
| --- | --- | ---: |
| `target/canonical-baseline.B6kFdI` | `5765709c84beb0bcc1bed0e158c51dabdf34bd982e7f7ac9810e3a16b14f1d57` | 2,024 matrix |
| `target/canonical-workloads.sCRUjV` | `5765709c84beb0bcc1bed0e158c51dabdf34bd982e7f7ac9810e3a16b14f1d57` | 256 workloads |
| `target/inline48.nJNbSC` | `b5563e4dfd101bfce5367972da8c0c9d58995845ff89ecafa1274462b52e1cc0` | 2,024 matrix + 256 workloads |
| `target/read-diagnostic-run.eTBfty` | `1457a52f4baedf8b428b42f9555331c67e0c799e91fb7ea88240471a5fa542bf` | 2,024 matrix |

All 6,584 successful measurements repeat exactly within each build, and both
engines return matching lengths. Workload assertions compare actual bytes against
the model. The first baseline workload attempt failed in host tuple decoding
before printing any measurements; its error log is retained. The corrected host
uses the same multi-result Candid decoder as the original matrix runner. A
Rustdoc lint was corrected before final Clippy qualification.

Native runner and Wasm benchmark Clippy, formatting and doc links pass. Full
release validation and native macOS execution are outside this investigation.
The initial retained change was workload coverage and evidence, with production
B-tree source identical to released 0.1.6. Experimental trees are neither
published nor adopted by IcyDB.

## Borrowed single-page key comparison

The follow-up removes a real allocation/copy inside our tree: binary search now
compares overflow keys of at most 4,072 bytes directly from the verified page
buffer. It checks the page kind and terminal pointer before passing the complete
key to the caller's comparator. Transactional searches read staged pages first,
as before. Inline and multi-page keys retain their prior paths. The public API,
format, full-page checksums and mutation publication contract are unchanged.

The starting build reproduces the baseline SHA-256 above exactly, so the retained
baseline measurements are reused. The new artifact is
`target/borrow-overflow-key.Uv5PJA/benchmark.wasm`, SHA-256
`14afb81dc15a33aced885f0060a1b60e8b259ba94dad4ea471da9648b31cb027`.
That directory retains its runner, both CSV suites and logs. All 2,280 measurements
repeat exactly; upstream counters and result lengths are unchanged against the
baseline. Mixed-workload results still match the independent heap model.

Four-round instruction totals, millions:

| Workload | Before | After | Change |
| --- | ---: | ---: | ---: |
| Index reads | 26.095 | 25.559 | -2.05% |
| Index fold put | 35.953 | 35.448 | -1.41% |
| Index fold delete | 76.581 | 75.476 | -1.44% |
| Index refill | 66.178 | 64.919 | -1.90% |
| Row reads | 38.327 | 37.702 | -1.63% |
| Row reverse key scans | 2.394 | 2.131 | -10.97% |

All 16 mixed profile/operation totals improve, by 0.14–10.97%. In the older matrix,
56/506 cases improve and 450 regress; the largest regression is 0.1164%, for a
one-MiB fixture's missing lookup. Dense 33-byte index-key lookup improves 2.29%;
the deep 262-byte-key row lookup improves 2.33%. This is retained for the mixed
workload gains with a small measured cost elsewhere, not claimed as a universal
speedup. Index operations still trail upstream, and large-read checksum costs
remain unresolved. Lowering allocation also changes later allocator history, so
scan improvements cannot all be attributed solely to copying fewer key bytes.

A consumer regression test injects payload corruption, authenticated wrong page
kinds and authenticated invalid chain endings into 33/4,072/4,073-byte keys.
Reads, membership checks, scans and mutations reject the damaged tree; failed
operations preserve all backing bytes. Restoring the injected page permits a
successful reopen/read/audit. Existing semantic-comparator, staging and differential
tests exercise mutation paths and ordering.

All 23 regular storage tests pass on Rust 1.99 and 1.85; four upstream tests and
the 320,000-operation extended campaign pass on 1.99. Native library/test and
Wasm benchmark Clippy pass. Full release and macOS qualification remain outside
this continuation. No IcyDB checkout, dependency selection or version is changed.

## Rejected branch-child allocation experiments

A further continuation tested allocating branch child-pointer vectors once during
node decoding, using the validated entry count. The initial implementation grows
an empty vector while decoding. Two candidates used either exact `count + 1`
capacity or power-of-two capacity (minimum four) matching the old final vector
capacity on the measured toolchains. Leaf nodes still allocate no child vector.
No persisted layout, integrity check or public API changed in either candidate.

The incoming IC Metrics 0.11.7 lockfile change is preserved. All other dependency
selections remain at the incoming state. A new baseline avoids mixing those
inputs with earlier 0.11.6 artifacts. Artifacts retain both CSV suites, logs and
runner; each candidate also retains its experimental `page.rs`:

| Attempt | Wasm SHA-256 |
| --- | --- |
| `target/node-capacity-baseline.GitnlI` | `d04602f08a325b10df51d5a78ffb3ffea415c37c3fb921661ac9bc84a4a0d256` |
| `target/node-capacity.mIqCpR` | `6d9baf6d5eb97fc04bd13b12efecefe33c2b59d2720847e7ec50ea83f69d57a4` |
| `target/node-capacity-rounded.5sNUN1` | `0f6bb15cece8f26a8aa1ae420c9fe4d1ceb7ef7f446c0d6733f6926f6ce3aac4` |

The exact-capacity candidate improves 500/506 older matrix cases and leaves six
unchanged, with unchanged upstream instruction counts. But the mixed workload
exposes a tradeoff: index reads/puts/deletes/refills improve 0.52–0.99%, while
forward row-value scans increase from 12,209,370 to 12,813,870 instructions across
four rounds (+4.95%). One scan round accounts for most of the regression despite
unchanged heap capacity. This is consistent with allocator history affecting
result-buffer growth; capacity counters alone do not establish the mechanism.

Matching the prior final vector capacity does not fix the regression: forward
row-value scans total 12,893,916 (+5.61%), and forward/reverse row-key scans regress
3.28/3.52%. Index mutation gains narrow to 0.44–0.82%. **Both candidates are
rejected.** The broadened workload prevents treating the older matrix's nearly
universal improvement as sufficient evidence for adoption. The rounded candidate
improves all 506 older-matrix cases, with unchanged upstream instruction counts.
All 6,840 measurements across the three builds repeat exactly, and the engines
return matching result lengths; the mixed workloads also verify actual bytes.

The exact-capacity candidate passes all 23 storage tests on Rust 1.99/1.85,
four upstream comparisons, native Clippy and benchmark Wasm Clippy. Both candidates
pass the mixed-workload model assertions. After rejection, `page.rs` matches its
incoming contents byte-for-byte, the rebuilt Wasm matches the baseline hash,
and the retained library passes its locked Rust 1.85 check. No new production
change, dependency update, release or IcyDB modification is retained by this
allocation investigation. See [dependency ownership](../upstreams.md#benchmark-dependency-resolution)
for the separate ICU/time inspection.
