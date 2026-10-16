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
