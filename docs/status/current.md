# Current handoff

The repository implements its own stable-memory B-tree in `crates/ic-storage`.
It supports custom encoded-key order, inline small payloads, overflow chains,
page reuse, fallible operations, lazy bidirectional ranges and structural audit.
The `ic-stable-structures` dependency supplies the Memory interface only; its
collection implementation is not used. IC Memory remains the allocation owner.

Read [the IcyDB boundary](../design/icydb-btree.md) and
[the package contract](../../crates/ic-storage/README.md) before changing the
format, comparator, allocator or mutation publication behavior. No IcyDB
checkout changes or retained-data conversion have been performed. Native model
and failure tests and Wasm checks do not establish canister rollback/recovery.

Shared Tooling 0.15.4 is recorded in the root snapshot. Development uses Rust
1.99.0; the library retains Rust 1.85.0 support. The root version remains 0.0.0
with a pending 0.1.0 release. No release or package upload has occurred.
