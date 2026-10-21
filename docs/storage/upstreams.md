# Upstream ownership

[Shared Tooling](https://github.com/dragginzgame/shared-tooling) owns the common
engineering baseline, tool installation, formatting, release coordination and
publication-attempt evidence. The root snapshot records the adopted revision.
Refresh it using the canonical exporter; never edit exported helpers locally.

[IC Memory](../../crates/ic-memory/README.md) owns allocation governance
and runtime memory handles. IC Storage consumes non-cloneable `RuntimeMemory`
leases and does not assign memory IDs, create managers or replace that authority.
Opening an allocation with a live lease is rejected by its runtime; dropping the
collection releases the lease. Hosts retain exclusive backing custody across
runtimes because lease tracking does not coordinate aliased independent runtimes.

DFINITY's `ic-stable-structures` owns the collection implementations and persisted
formats wrapped by `ic_storage::adapter`. Storage consumes the exact types through
IC Platform's optional `stable-structures` boundary. Consumers use the supported adapter
and obtain admitted allocations through IC Memory. Public wrappers expose neither
raw memory, collection extraction nor unrestricted upstream collections. The only
public Rust module is `ic_storage::adapter`; its collection wrappers, iterator
aliases, error and codec re-exports are the supported integration surface.
The exported `Memory` trait supports generic bounds, not allocation authority.
`BoundedCell` checks declared encoding bounds around the upstream cell; record
schemas, decoding semantics and recovery remain consumer-owned. Map reservation
accepts caller-owned page budgets and makes no node-layout entry-capacity promise.

Shared Tooling's external-IC dependency policy requires the substrate to pass
through Platform. Both packages select published Platform 0.6.5 with defaults
disabled and the `stable-structures` feature. Existing manager identities are
provided by [Platform #45](https://github.com/dragginzgame/ic-platform/issues/45).
The direct-dependency allowances and exact-declaration exception are retired;
production admission now checks the exact registry 0.7.2 package identity and
singleton Memory/Platform/substrate graph. Declaration policy forbids direct
substrate edges in either workspace package.

This adoption is tracked in
[#130 — Route stable-memory substrate dependencies through IC Platform](https://github.com/dragginzgame/ic-persistence/issues/130).
These are existing type re-exports; Memory retains layout/growth checks and
allocation authority, and collection bytes and public type identities stay intact.

`BTreeMapRegion` retains an existing `Rc<RuntimeMemory>` for an exact range inside
that allocation. IC Storage owns bounded region IO and operation-local upstream
tree loading; it neither assigns another allocation nor exports a raw-memory
constructor. The application owns disjoint frame access, framing/namespace
admission and capacity budgets. All views retain the same lease and reload roots
per operation; ordered key scans return owned snapshots. No IC Memory API or
dependency change is needed. See the [retained-tree guide](../../crates/ic-storage/README.md#retained-trees-inside-a-shared-allocation).

The dependency checker applies consumer-owned policy in
`ci/cargo-dependency-boundaries.json`; its implementation is included in the root
Shared Tooling snapshot. It checks this workspace's dependency declarations, not
sibling adoption or runtime handle construction. The adapter constructors enforce
the handle boundary; collection lease and reopen tests qualify that integration.

IcyDB owns application codecs, comparator semantics, schema authority and
application recovery. The former independent B-tree and its benchmark have been
removed. Their [archived research](../archive/private-btree/README.md) is historical
and supplies no reader or conversion for custom-format bytes.

## Test fixtures

The adapter's lease and reopening tests use this workspace's IC Memory
`testing::fixture_runtime`, contributed in
[IC Memory PR #120](https://github.com/dragginzgame/ic-memory/pull/120). The caller
selects backing, owner, namespace, stable keys and default 128-page bucket geometry.
IC Memory owns ordinary admission and ledger bootstrap. The fixture feature is
selected only through the library's dev dependency; production builds use the
ordinary IC Memory dependency. These tests exercise collection ownership of
leases and persisted round trips. Declaration validation, allocation placement
and ledger recovery remain IC Memory's implementation and test responsibilities.

## Host operations

[IC Host Tooling](https://github.com/dragginzgame/ic-host-tooling) owns reusable
artifact, filesystem and process operations. The retired benchmark was this
workspace's only host-helper and IC Testkit consumer, so its dependencies have
been removed. Reuse those owners if a future implementation needs their contracts.
Select compatible registry versions in the root workspace catalog and inherit
them in members; mutable sibling paths are not release dependencies.

Release metadata rendering uses IC Build through the Memory-hosted repo-tool
example. The root `ci/release-tools.env` selects the published bootstrap editor;
`make install-release-tools` prepares it explicitly. Both package archives and
versions participate in the same source-bound receipts and rollback transaction.
See [release preparation](../../RELEASING.md) for publication order and recovery.
