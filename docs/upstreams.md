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
