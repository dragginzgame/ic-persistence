# Current handoff

## Scope and ownership

The workspace contains the `ic-storage` governed collection adapter only.
IC Memory owns allocation, runtime bootstrap and exclusive leases; upstream
`ic-stable-structures` owns collection algorithms and persisted formats. IcyDB
owns application codecs and recovery. See [current ownership](../upstreams.md).

The private B-tree and benchmark are retired. Their
[archived research](../archive/private-btree/README.md) is historical evidence;
no reader converts their incompatible bytes into upstream collection formats.
Governed region views and the adapter's lease/reservation contracts are described
in the [package guide](../../crates/ic-storage/README.md).

## Current source and pending work

The task started at released 0.5.8, commit `155ef68`, with a pending Shared
Tooling 0.25.5 refresh. The incoming Cargo.lock selection of IC Memory 0.49.3
is preserved without modification or new runtime
qualification. Package versions, the Git index and release refs remain
maintainer-owned.

The [root snapshot](../../.shared-tooling.snapshot) selects Shared Tooling
0.25.8 at `0a2d2ee3b5e0ae77755c2915ac28836d4d5ba359`, matching observed remote
main on 2026-10-20. The canonical exporter refreshed the same 104-file closure.
Failed Cargo metadata checks now retain diagnostics, original root inputs and
separate Cargo/logger statuses without retries. Release preparation accepts both
numbered pending heading styles, replacing an optional `Unreleased` marker with
the saved UTC date. Tool catalogs, release editor and compiler selections remain
unchanged; no reinstall is needed.

Pending compatible patch 0.5.9 records the tooling adoption and early lint gate.
Published notes remain intact. Library APIs, collection formats and dependency
selections are unchanged by this adoption.
The repository description already matches the current adapter scope.
There are no owned application endpoints or local imported runtime/cryptographic
implementations requiring migration under the updated encoding/maintenance rules.

## Release and validation

Authorized fresh release preparation installs and checks common tools and the
selected IC Build 0.18.0 metadata editor. Standalone preflight remains offline
and check-only. The local adapter supplies the admitted editor path and a 1 MiB
per-document allowance; no manual shell export is required. The six-case
`test-release-preflight` fixture remains part of `check-release-commands`.
The complete gate now runs the unchanged workspace/all-targets strict Clippy
selection before product builds, docs, tests and packaging. Publication commands
also run that gate after source/tag admission and before creating an attempt.

Focused Linux checks pass: snapshot integrity, links, licenses, pins, dependency
declarations, release dispatch, six preflight scenarios, offline tool/editor
admission, formatting, shell syntax, ShellCheck and diff hygiene. Disposable
substitutions verify the actual gate under parallel Make, success/failure ordering
for both publication commands, both pending heading spellings, and retained Cargo
metadata failure evidence without retry. These do not execute product Clippy,
full CI, a live release or publication. Native macOS remains unqualified here.

## Remaining upstream-dependent work

The direct substrate edge remains migration debt under Platform policy; see
[Platform #30](https://github.com/dragginzgame/ic-platform/issues/30) and
[PR #32](https://github.com/dragginzgame/ic-platform/pull/32). Before adopting a
published interface, preserve Storage's public codec names and original type
identities. The previous
[compatibility review](https://github.com/dragginzgame/ic-platform/pull/32#issuecomment-6268357398)
records the missing exports and source-copy qualification. Producer status and
consumer runtime compatibility were not requalified by this tooling adoption.

[Issue #12](https://github.com/dragginzgame/ic-storage/issues/12) owns discard-value
map mutations. Require a genuine released primitive and consumer IC measurements;
dropping an ordinary returning mutation's result does not avoid its old-value
work. Upstream release/PR status was not refreshed in this pass.

The complete preceding handoff and completed qualification history are preserved
at [commit a2fcb59](https://github.com/dragginzgame/ic-storage/blob/a2fcb59/docs/status/current.md).
Prior source proofs and the proposed Platform codec patch remain under
`/tmp/ic-storage-next-improvements.BJsY3o/`; prior tool qualification is under
`/tmp/ic-storage-shared-0253.dCVYCb/`. The preceding refresh's preservation evidence
is under `/tmp/ic-storage-shared-0255.ZIwfCQ/`; this adoption's fixtures and incoming
state are under `/tmp/ic-storage-shared-0258.bfnYBW/`. These are local paths, not releases
or substitutes for owning GitHub issues. Sibling checkouts and prior artifacts
are preserved.
