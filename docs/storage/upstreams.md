# Collection engine provenance

Storage owns allocation governance, backing custody, named directory and public
collection adapters. Its private `engine` module copies the collection algorithms
from published `ic-stable-structures` **0.7.2**. The maintainer explicitly selected
this source baseline instead of another independently maintained implementation.
There is no production dependency or public re-export of ICSS.

[The source inventory](icss-source.json) records original source SHA-256 hashes.
The relevant map/node/allocator/iterator, BaseVec/Vec, Cell, address and helper
code is copied under `crates/ic-storage/src/engine/`. Each copied file retains
DFINITY attribution and an Apache-2.0 notice; the package includes LICENSE-APACHE.
Storage-authored code remains MIT. This is adapted source, not an unchanged
vendored executable snapshot.

## The two collection layers

- Private `engine` owns one implementation of each algorithm. Set uses the same
  map engine with unit values; region views use that map over a bounded lease.
- Public `adapter` preserves successful-call API shapes and requires admitted
  `RuntimeMemory` handles. Its private bridge connects Storage's existing
  `Memory`, `Storable` and `Bound` contracts to the copied private traits.

The old independent tree, block allocator, record envelopes, node cache, Vector
windows and mutation epochs are deleted. No alternative production engine stays
selected or maintained. Directory allocation remains Storage's implementation:
ICSS's MemoryManager, ID assignment and unrestricted backing constructors are
not copied. Physical access still passes through checked Storage handles.

## Deliberate adaptations

The algorithms, current node/overflow layouts, fixed-stride Vector slots and
scalar iterators follow 0.7.2. Changes are limited to integration and safety:

- Private module paths and checked backing/codec views; existing application
  codecs remain unchanged. Storage unsigned codecs are little-endian, whereas
  ICSS's default unsigned codecs are big-endian. Same-image interoperability
  requires matching codecs, bounds, ordering and collection framing.
- Explicit create/load/reset admission. Remove upstream implicit initialization
  and old B-tree/node migration readers. Only current explicit-page-size BTR2,
  SVC1 and SCL1 images are admitted. No previous Storage image is read or reset.
- Cell admits the host's byte ceiling before buffer allocation. Vector checks
  a variable slot length against its declared bound before allocating payload.
- A persisted allocator allocation flag is read as u8, checked, then interpreted,
  preserving valid bytes without constructing an invalid Rust bool on corruption.
- Native mutation unwind guards poison the shared owner after a panic. They add
  no Wasm journal or runtime work; IC rollback remains the atomicity mechanism.
- An owned raw-value iterator adapter retains Storage's public MapEntry API.
  Skipping delegates to the engine before materializing the selected entry.
- Remove unused APIs and upstream tests; qualify the copied subset using Storage
  models and the separately published reference. Rust 2024 unsafe blocks, checked
  conversions, lint fixes and narrowly explained expectations are mechanical
  adaptations, not performance enhancements.

Codec and engine failures trap. Framing admission and explicit reservations can
still return typed errors; Result signatures do not promise pre-write rejection.
There are no collection checksums. Existing Storage collection images require an
explicit host rebuild/reset or fresh allocations. The directory format and its
integrity checks are unchanged. See [the collection contract](../../crates/ic-storage/README.md).

## Qualification and research controls

The independent workspace retains published ICSS 0.7.2 through its Platform 0.17
benchmark facade and optional Git `b5d1650` as separate research controls. Neither
enters production. Same-image tests exchange current collections using matching
application codecs and governed Storage backing. Installed lifecycle tests and
matched IC benchmarks establish behavior and costs separately.

Historical audits and measurements describe the engines selected when they ran;
they do not establish current performance. The copied baseline deliberately
removes earlier local optimizations so later changes can be compared cleanly.
No universal improvement or ICSS default-codec compatibility is claimed.

IC Platform supplies Wasm stable-memory system access; IC Wire supplies bounded
directory encoding and optional application codecs. Testkit owns server lifecycle;
Shared Tooling owns its reviewed engineering snapshot, unchanged by this port.
Hosts retain lifecycle, comparators, schema and recovery decisions. Sibling code
and retained images have not been migrated by this change.
