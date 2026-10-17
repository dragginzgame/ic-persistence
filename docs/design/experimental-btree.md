# Private page-based B-tree

The independent B-tree is private implementation work. It is compiled for
library qualification tests and, from the same source, the unpublished canister
benchmark. No Cargo feature exposes it to downstream crates. IcyDB and Canic
continue to use the governed upstream collections in `ic_storage::adapter`.
Public adoption requires a separate API decision and consumer qualification.

The private governed wrapper still consumes an exclusive IC Memory runtime
handle. Its engine, configuration, comparator, errors, entries and ranges are
not exported from the crate root or the adapter. Generic memory access remains
an internal fault-injection and differential-testing facility.

Keys up to 32 bytes and values up to 16 bytes are inline in tree nodes. Larger
payloads use checksummed chains. Nodes have at most 31 entries, with split,
borrow, merge and root-shrink operations. Deleted nodes and payload pages return
to a persisted free list; stable memory's physical capacity does not shrink.
Replacing an existing value preserves tree topology and reuses its overflow
pages. Shrinking frees excess pages; growing allocates only the extra pages
needed. Same-size and shrinking replacements do not require backing growth.
Replacement reuses the located node and collects the returned previous value
during its overflow-chain traversal, avoiding a second search and payload read.
Default limits accept keys up to 128 KiB and values up to 4 MiB. Callers select
and retain exact limits when creating/opening a store. Values are opaque bytes;
application codecs remain responsible for their meaning and validation.

Custom `Comparator` implementations own encoding-aware order. Their 16-byte
identity is persisted and checked on open. Ordering must remain deterministic,
immutable and total; equal keys must have identical encodings. Change the
identity whenever ordering or accepted encoding changes. Identity equality is
not proof that two binaries implement the same comparator.

Ranges accept included, excluded and unbounded endpoints, support reverse and
interleaved traversal, and retain tree paths rather than materializing a full
store. Key-only scans and `contains_key` avoid external value-payload reads;
inline value bytes share the node page. Iterator errors terminate traversal.
`validate()` explicitly audits all live/free pages, checks ordering and counts,
and rejects duplicate ownership, cycles, orphan pages and uneven tree depth.
Opening checks the header/root; it does not run that full scan.

## Persistence boundary

Each tree requires exclusive ownership of one memory region, with one live
handle. Do not mutate that region through another handle or retained memory
clone. Creation requires zero-sized memory and rejects existing data. Opening
requires the exact format, configuration and comparator identity.

Mutations stage touched pages and payloads in heap memory, then reserve backing
capacity before the first write. Returned validation/capacity errors leave the
published tree bytes unchanged. Successful growth can reserve unused capacity.
Once writes begin, the `Memory` implementation must complete them or trap within
an atomic IC message that rolls its writes back. Do not catch a native backing
panic and continue using the tree. This is **not filesystem crash recovery**:
a process crash or partial non-transactional write can leave mixed pages. The
checksums detect accidental corruption; they are not cryptographic authentication.
An application journal, transaction coordinator and multi-store atomicity remain
outside this collection.

This format is new and incompatible with DFINITY stable B-tree bytes. Adoption
must explicitly select fresh memory and resolve retained-data/rebuild obligations
under the application's recovery contract. Never reopen old bytes as this tree,
erase an allocation ledger, or infer that pre-1.0 stored data is disposable.

Native tests cover differential mutations, splits/merges, page reuse, bounded
bidirectional traversal, large rows/keys, semantic ordering, growth rejection
and corruption. Wasm compilation is checked. These checks do not establish IC
message rollback, upgrade recovery, or `IcyDB` integration/performance qualification.

