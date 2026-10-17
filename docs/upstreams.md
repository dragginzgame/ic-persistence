# Upstream ownership

[Shared Tooling](https://github.com/dragginzgame/shared-tooling) owns the common
engineering baseline, tool installation, formatting, release coordination and
publication-attempt evidence. The root snapshot records the adopted revision.
Refresh it using the canonical exporter; never edit exported helpers locally.

[IC Memory](https://github.com/dragginzgame/ic-memory) owns allocation governance
and runtime memory handles. IC Storage consumes non-cloneable `RuntimeMemory`
leases and does not assign memory IDs, create managers or replace that authority.
Opening an allocation with a live lease is rejected by its runtime; dropping the
collection releases the lease. Hosts retain exclusive backing custody across
runtimes because lease tracking does not coordinate aliased independent runtimes.

DFINITY's `ic-stable-structures` owns the collection implementations and persisted
formats wrapped by `ic_storage::adapter`. Only the infrastructure package depends
directly on the substrate. Applications use the supported adapter and obtain
admitted allocations through IC Memory. Public wrappers expose neither raw memory,
collection extraction nor unrestricted upstream collections. The dependency
checker applies consumer-owned policy in `ci/cargo-dependency-boundaries.json`;
its implementation is included in the root Shared Tooling snapshot.

IcyDB owns application codecs, comparator semantics, schema authority and
application recovery. The former independent B-tree and its benchmark have been
removed. Their [archived research](archive/private-btree/README.md) is historical
and supplies no reader or conversion for custom-format bytes.

## Test fixtures

The adapter's lease and reopening tests use published IC Memory 0.48's
`testing::fixture_runtime`, contributed in
[IC Memory PR #120](https://github.com/dragginzgame/ic-memory/pull/120). The caller
selects backing, owner, namespace, stable keys and default 128-page bucket geometry.
IC Memory owns ordinary admission and ledger bootstrap. The fixture feature is
selected only through the library's dev dependency; production builds use the
ordinary IC Memory dependency.

## Host operations

[IC Host Tooling](https://github.com/dragginzgame/ic-host-tooling) owns reusable
artifact, filesystem and process operations. The retired benchmark was this
workspace's only host-helper and IC Testkit consumer, so its dependencies have
been removed. Reuse those owners if a future implementation needs their contracts.
Select compatible registry versions in the root workspace catalog and inherit
them in members; mutable sibling paths are not release dependencies.
