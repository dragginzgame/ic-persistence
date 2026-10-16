# Upstream ownership

[Shared Tooling](https://github.com/dragginzgame/shared-tooling) owns the common
engineering baseline, tool installation, formatting, release coordination and
publication-attempt evidence. The root snapshot records the adopted revision.
Refresh it using the canonical exporter; never edit exported helpers locally.

[IC Host Tooling](https://github.com/dragginzgame/ic-host-tooling) owns reusable
host operations: `ic-host-fs` for bounded filesystem operations and locks,
`ic-host-artifacts` for artifact streams, `ic-host-process` for admitted process
execution, and `ic-host-tools` for IC host tooling. Reuse those packages when
implementation needs their contracts instead of recreating host infrastructure.
Add selected compatible
registry versions to the root workspace dependency catalog, inherit them in
members and commit their exact resolution in Cargo.lock. Mutable sibling paths
are not release dependencies. Product storage contracts remain owned here.

[IC Memory](https://github.com/dragginzgame/ic-memory) owns allocation governance
and runtime memory handles. IC Storage does not create memory IDs or replace
that authority. DFINITY's `ic-stable-structures` supplies the common Memory trait;
IC Storage implements its own collections and incompatible persisted format.
IcyDB owns typed key codecs, comparator semantics and application recovery; see
[the consumer boundary](design/icydb-btree.md).

[IC Testkit](https://github.com/dragginzgame/ic-testkit) owns canister performance
counters, typed simulator calls and the managed PocketIC server lifecycle used by
`ic-storage-bench`. Its native runner uses IC Host FS for bounded artifact reads
and hashes. These dependencies belong to the non-published benchmark package;
the `ic-storage` library still depends only on the shared Memory substrate.
