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
not load overflow values until requested. Replacement can temporarily need both
old and new payload space before freeing the old chain.

The format has its own magic and never claims compatibility with DFINITY B-tree
pages. IcyDB adoption requires its own comparator/codec adapter, propagation of
fallible operations through overlay and recovery boundaries, a retained-data
transition decision, native/PocketIC qualification and workload benchmarks.
These are integration acceptance requirements, not claims established by this
repository's native model tests or Wasm compilation.
