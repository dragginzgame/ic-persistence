# Current handoff

The maintainer retired the private B-tree in favor of a different plan. Its
engine, custom-format qualification tests, private aliases and unpublished
comparison benchmark have been removed. The workspace now contains only the
governed upstream collection library. Benchmark targets and benchmark-only
dependencies are removed.
The supported adapter still consumes IC Memory runtime handles and preserves
upstream persisted formats. No replacement engine is implemented by this batch.

The adapter's allocation lease and reopening test is retained under
`src/adapter/tests.rs`, using published IC Memory 0.48 fixture support only as a
dev dependency. The former engine's design documents, performance investigation,
previous handoff and ownership notes are retained in the
[historical archive](../archive/private-btree/README.md). Existing build artifacts,
benchmark results and release failure evidence are untouched. Custom-format bytes
are not reinterpreted as upstream collections.

Pending notes select 0.5.2. Package versions, Git index and refs remain
maintainer-owned. Shared Tooling remains at the reviewed 0.23.0 snapshot. The
current package description reflects the governed collection surface.

The adapter test passes on Rust 1.99 and 1.91; all eight Rustdoc examples pass.
Native all-target Clippy, warning-free Rustdoc, Wasm compilation, formatting,
documentation links, dependency boundaries and release-command checks pass.
The lockfile shrinks from 297 packages to 22 without changing retained package
versions or checksums. Package source listing excludes retired engine/benchmark
files. The coordinated release graph contains only the library.

Dependency-pin validation passes on an isolated copy of the final source with
its own index. In the real checkout the canonical checker refuses the removed
benchmark manifest until its deletion is staged; the maintainer-owned index is
untouched. Full release CI and macOS checks are not run locally. No retained
data or old build/benchmark evidence is deleted.
