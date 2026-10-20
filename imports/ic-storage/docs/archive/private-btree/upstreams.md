# Upstream ownership

> Historical record retained after removal of the private B-tree. Commands,
> APIs and ownership statements below describe the retired implementation.
> See the [archive context](README.md) and [current ownership](../../upstreams.md).

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
IC Storage implements its own B-tree and incompatible persisted format, and
wraps upstream collections through `ic_storage::adapter`. Both infrastructure
crates may depend on upstream privately. Application and benchmark collection
code uses IC Storage; runtime bootstrap uses IC Memory.
IcyDB owns typed key codecs, comparator semantics and application recovery; see
[the consumer boundary](icydb-btree.md).

[IC Testkit](https://github.com/dragginzgame/ic-testkit) owns canister performance
counters, typed simulator calls and the managed PocketIC server lifecycle used by
`ic-storage-bench`. Its native runner uses IC Host FS for bounded artifact reads
and server-file hashing, and IC Host Artifacts for hashing the loaded Wasm bytes.
The reported Wasm identity therefore describes the bytes passed to installation
without a second pathname read. The native runner directly selects IC Host
Artifacts and FS 0.27, locked to 0.27.1. IC Testkit 0.52 retains its own compatible
Host 0.26 dependencies; the runner does not construct or downcast those Host error
types across versions. These dependencies belong to the non-published benchmark package;
the `ic-storage` library depends on IC Memory and the upstream substrate.

## Benchmark dependency resolution

The workspace deliberately has two Rust floors: 1.91 for the library and 1.99
for the benchmark. Cargo resolver 3's Rust-version fallback can choose versions
compatible with the lowest workspace floor even for dependencies used only by a
higher-floor member; see [Cargo's mixed-workspace resolution rules](https://doc.rust-lang.org/cargo/reference/resolver.html#rust-version).
An older host-only dependency is therefore not necessarily an upstream pin or a
requirement of the library's actual dependency graph.

On 2026-10-16, a registry-refreshed dry run of
`cargo update --dry-run -p time -p icu_normalizer -p icu_properties -p idna_adapter`
selected no changes, explicitly reporting Rust 1.85 compatibility. Repeating
with `--ignore-rust-version --dry-run --offline` selected ICU 2.3.0 (provider
2.3.1), idna_adapter 1.2.2, time 0.3.55 and their companion crates. Registry
metadata declares Rust 1.88 for those ICU/time versions and 1.86 for the adapter.
That inspection used ICU 2.1.1/2.1.2, idna_adapter 1.2.1 and time 0.3.45.

`cargo tree --workspace --locked -i icu_normalizer` traces ICU through
idna_adapter, idna, url, reqwest, PocketIC and IC Testkit into the native benchmark
runner. `time` reaches the same runner through tracing-appender/subscriber.
Neither appears in `ic-storage` or the benchmark's Wasm dependency graph.
The maintainer subsequently selected Rust 1.91 as the library floor. A targeted
update under normal resolver fallback then selected ICU 2.3.0 (provider 2.3.1),
idna_adapter 1.2.2 and time 0.3.55 with their companion crates. No resolver
override is needed. The incoming IC Metrics 0.11.7 selection is preserved.
The library's locked MSRV gate now uses Rust 1.91; the native benchmark remains
on Rust 1.99. Earlier benchmark artifacts retain their original dependency
identities rather than being relabelled with these newer host dependencies.

Qualification: Rust/Cargo 1.91.0 passes `make msrv` and all 32 regular library,
storage and upstream tests. Rust 1.99 builds the native runner and release Wasm;
native library/test/runner and Wasm benchmark Clippy pass. New MSRV-aware lints
are addressed with let-chains and `as_chunks::<8>()`, preserving checksum bytes.
The host-only update attempt `target/msrv191-host-deps.5o6Qd3` repeats the prior
256-row workload CSV exactly. After the lint changes, final attempt
`target/msrv191-final.yauikU` passes all 256 workload measurements and model
checks with exact repeats. Its Wasm SHA-256 is
`227286353170520bdac6b0ddcb9400b623bdbf69fa1f6687f7e3325c3b73c7d8`;
both attempts retain their native runner, lockfile, Wasm, CSV and log. This is
Linux qualification; macOS CI is updated but not executed locally. Formatting,
dependency pins and documentation links pass; the full release gate is not run.

## Collection ownership boundary

IC Storage consumes IC Memory; IC Memory must never depend on IC Storage.
Public collection wrappers accept `RuntimeMemory` and expose neither their
inner collection nor memory extraction. Generic backing memory is retained for
runtime infrastructure and qualification. This is a breaking consumer API cut;
IcyDB must obtain admitted handles and use the storage facade for collections.
Existing upstream formats remain upstream formats; our tree's format is unchanged.

The dependency guard permits the direct substrate dependency only in this
workspace's `ic-storage` package, including renamed dependency declarations.
It is a local guard, not a claim that other repositories have been migrated.

IC Memory 0.46 supplies non-cloneable allocation leases. Opening an allocation
with a live lease returns `RuntimeOpenError::AlreadyOpen`; dropping the owning
collection releases it. Collection constructors consume the lease, including
two distinct allocations for a log. The runtime keeps lease state alive while
handles outlive it. Typed custom-tree constructor errors drop their consumed
handle and permit retry. This ownership boundary replaces the previous caller-
enforced clone discipline; it is a breaking public dependency cut for IC Storage.

Lease tracking is scoped to one runtime. Hosts must keep exclusive custody of
backing memory until that runtime and every outstanding lease have been dropped.
Do not open another runtime or write through retained backing aliases during
that lifetime. IC Memory exports only `Memory`, `DefaultMemoryImpl` and
`VectorMemory` for infrastructure setup; its broad upstream module is removed.
IC Storage's own collection wrappers remain the application-facing surface.


## IC Memory 0.45 and IC Testkit 0.50 adoption

The incoming dependency selections are preserved: IC Memory 0.45, IC Host FS
0.24 and IC Testkit 0.50. Allocation requests now contain authority and stable
key only; the adapter tests and benchmark no longer pass removed schema metadata.
The native runner explicitly depends on IC Testkit Candid 0.50 for `DecodeLimits`.
All seed and measurement calls admit at most 256 reply bytes, 32 header bytes,
10,000 decoding work units, no type-table entries and no skipped values. Replies
are primitive booleans or four integer counters, never stored row payloads.
These limits are qualified by the governed mixed-workload run; they do not change
canister measurement regions or storage formats. Runtime libraries remain free
of this native decoding dependency.


## Dependency-boundary helper adoption

The generic checker from
[Shared Tooling PR #257](https://github.com/dragginzgame/shared-tooling/pull/257)
is now included in the root Shared Tooling 0.23.0 snapshot. The canonical
exporter's snapshot takeover verified and retired the separate contribution
bundle and its receipt. Future refreshes use only the root manifest.
`make check-storage-boundary` runs the shared checker with consumer-owned policy
`ci/cargo-dependency-boundaries.json`. Rules cover canonical names of normal,
build, dev and target-specific direct dependencies, including renames. Upstream
owns rejection/schema fixtures; this consumer checks its real Cargo metadata
and root snapshot integrity.


## Fixture-runtime ownership

Published IC Memory 0.48 includes `testing::fixture_runtime` from
[IC Memory PR #120](https://github.com/dragginzgame/ic-memory/pull/120). Collection
tests and the unpublished benchmark use it instead of constructing allocation
authorities, requests and bootstrap policy locally. Callers still select their
backing, owner, namespace, keys and default 128-page bucket geometry. The helper
returns an ordinary admitted runtime without initializing any collection.

The library enables `test-support` only as a dev dependency; the benchmark
enables it only for its Wasm memory dependency. Production IC Storage keeps the
ordinary IC Memory dependency and no fixture feature. No Git/path override or
new memory authority is introduced.


## IC Memory 0.46 lease qualification

The incoming registry update is adopted without exposing raw collection APIs.
Public integration tests verify concurrent-open refusal and release-on-drop for
our tree, every adapted upstream collection and both log allocations. Another
regression verifies that invalid configuration during create/open releases the
custom tree's lease and permits a later valid open. Compile-fail examples use
the new root backing exports and reject raw memory, extraction, implicit inner
collection access and lease cloning. The valid README example passes too.

The fixture helper in PR #120 is updated against released 0.46, retaining its
new passive fixture exports and existing cell error categories. Its ten focused
tests pass on Rust 1.91, with test-support Clippy and warning-free Rustdoc.
Local adoption remains gated on that helper's registry release. IC Storage's
numbered pending notes are 0.4.0 because public constructors now require 0.46
handle types and non-cloneable lease semantics; package versions are unchanged.

## Supported adapter errors

The stable collection adapter exports `LogWriteError` for the supported log
append API. It no longer exports upstream cell initialization/value errors,
vector initialization errors, log initialization errors or `NoSuchEntry`, because
none is returned by the supported wrapper methods. This is a hard cut in the
pending 0.4.0 API. Upstream initialization and panic contracts are unchanged;
removing error aliases does not introduce new fallible collection operations.
A local search found no uses of these exported names in the IcyDB crate sources.
