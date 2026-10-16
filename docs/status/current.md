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

Shared Tooling 0.15.4 is recorded in the root snapshot. Its revision
`879eb2b021522252a67e908dbc217a0f4efb2a6f` matches remote HEAD as checked on
2026-10-16; all 99 exported files pass snapshot verification. Uncommitted sibling
changes are not adopted. Development uses Rust 1.99.0; the library retains
Rust 1.85.0 support.

The pending 0.1.1 patch makes value replacement reuse existing overflow pages
without splitting tree nodes. It preserves the public API and stored format.
Regression coverage includes full-root replacement, overflow/inline transitions,
reopening, page reuse and rejected growth without partial writes.

Additional 0.1.1 qualification exercises 700 length-framed keys against a decoded
key model through splits, mixed inline/overflow replacements, deletions, bounded
interleaved scans and reopening. The focused lifecycle test passes on Rust 1.99
and 1.85; the integration test target passes Clippy. No production change was
needed for this coverage. This remains a synthetic comparator, not IcyDB's codec.
The accepted IcyDB transition is a hard cut without a legacy reader; retained-data
disposition and canister qualification remain required before adoption.

Cargo.toml and the local Cargo.lock package identity now both select 0.1.0.
The stale 0.0.0 lock entry caused locked release preflight to refuse; it was
synchronized with the shared lockfile helper without changing external dependency
selections. Locked offline Cargo metadata and fetch now pass in this checkout.
Earlier B-tree validation used an isolated copy to accommodate the then-unfinished
metadata; that limitation is now resolved. No release, commit, tag or upload was
executed as part of this repair.
