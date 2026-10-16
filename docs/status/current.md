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

The released 0.1.1 patch makes value replacement reuse existing overflow pages
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

Cargo.toml and Cargo.lock now both select released 0.1.1, tagged at `19d53cc`.
The earlier lockfile mismatch is resolved; validation runs in the real checkout.

Pending 0.1.2 reuses the located node and reads the previous overflow value once
during replacement. Native interface read traffic is halved for the index,
journal-sized value and row fixtures; writes and reserved capacity are unchanged.
The reproducible command and before/after measurements are in the
[IcyDB design](../design/icydb-btree.md#native-memory-traffic-qualification).
The API, persisted format and staged-write failure contract remain unchanged.
Validation: 16 storage tests pass on Rust 1.99 and 1.85; the opt-in traffic
fixture, Clippy, Wasm compilation, rustdoc, formatting and doc links also pass.
This does not establish canister instruction costs, heap usage or IcyDB integration.
Release versions, commits, tags and publication remain maintainer-owned.

Direct upstream qualification now compares both persisted trees with a heap
model: 8,000 regular and 320,000 extended seeded operations passed, plus boundary
ranges, maximum payloads and rejection of actual upstream-format memory. The
four regular tests pass on Rust 1.99 and 1.85; the extended campaign passed in
release mode on 1.99. Traffic fixtures compare unbounded upstream storage and a
bounded index layout. Small index entries cost more bytes/capacity in our tree,
while large-value replacements use less traffic than the unbounded upstream
fixture. Fewer Memory calls do not prove fewer IC instructions; actual IcyDB
schemas and bounded large-value comparisons remain unqualified. See the design
document for reproducible commands, measurements and limitations.
