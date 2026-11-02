# Upstream ownership

Storage owns the allocator, backing trait, codec trait and Cell, Map, Set, Vector
and map-region implementations. No imported `ic-stable-structures` source,
re-export, feature or selected dependency remains. Production graph admission
forbids that package. The replacement is a complete retained-format hard cut.

IC Platform supplies Wasm stable-memory system access. IC Wire supplies bounded
directory and collection-record encoding. Storage owns their framing, checksums,
resource limits and allocation/collection invariants. Hosts own application
codecs, comparators, schema compatibility, lifecycle and recovery decisions.
See [Storage](../../crates/ic-storage/README.md) and
[allocation](../../crates/ic-storage/ALLOCATION.md).

Collections accept only admitted `RuntimeMemory` leases. `BTreeMapRegion` uses an
`Rc<RuntimeMemory>` over a caller-selected page range and reloads the owned tree
root per operation. Hosts must admit disjoint framing and update old `BTR`/`BTA`
checks to the current `ICMAP001` contract. No region allocates another identity.

Shared Tooling owns the reviewed engineering baseline, formatting, setup and
release runner. The snapshot remains unchanged by this rewrite. Build and Host
own reusable native metadata, filesystem, process and archive operations;
Storage's repo-tool retains consumer release policy and qualification receipts.
Testkit owns server selection/lifecycle; the independent runner owns installed
Storage and adapter scenarios. None of those native tools enters the production
canister graph. See [adoption boundaries](../shared-tooling-adoption.md).

The test-support `fixture_runtime` performs the same namespace admission and
bootstrap as production. Tests choose their own backing, stable keys and
`StorageConfig`. Sibling repositories remain separately responsible for adopting
the changed public types and explicit reset requirement.
