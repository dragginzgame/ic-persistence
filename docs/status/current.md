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

The task started at released 0.5.7, commit `a2fcb59`. The incoming Cargo.lock
selection of IC Memory 0.49.1 is preserved without modification or new runtime
qualification. Package versions, the Git index and release refs remain
maintainer-owned.

The [root snapshot](../../.shared-tooling.snapshot) selects Shared Tooling
0.25.3 at `c55c1065c0061455dab7f0e2be8ddd04fdeb9910`, matching observed remote
main on 2026-10-20. The canonical exporter refreshed the same 104-file closure.
Common tool versions, the release editor pin and the development/MSRV selectors
are unchanged. Selected Cargo tools now support explicit feature sets and give
more precise admission/setup diagnostics. This consumer requests no extra tool
features. Rerun `make install-tools` and `make install-release-tools` after
adoption because the shared build-policy identity changed; prior generations
remain retained. Both setup commands succeeded on this Linux host.

The numbered pending notes select compatible patch 0.5.8 for contributor tooling;
this adoption changes no library API, collection format or dependency selection.
The repository description already matches the current adapter scope.
There are no owned application endpoints or local imported runtime/cryptographic
implementations requiring migration under the updated encoding/maintenance rules.

## Release and validation

Authorized fresh release preparation installs and checks common tools and the
selected IC Build 0.18.0 metadata editor. Standalone preflight remains offline
and check-only. The local adapter supplies the admitted editor path and a 1 MiB
per-document allowance; no manual shell export is required. The six-case
`test-release-preflight` fixture remains part of `check-release-commands`.

Focused qualification covers snapshot integrity, source licenses, dependency
pins and declarations, release dispatch and the six substituted preflight cases.
Prepared-tool admission, formatting, documentation links, shell syntax,
ShellCheck and diff checks also pass on Linux. Full CI, a live release,
publication and native macOS qualification are outside this pass.

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
`/tmp/ic-storage-next-improvements.BJsY3o/`; tooling adoption evidence is under
`/tmp/ic-storage-shared-0253.dCVYCb/`. These are local evidence paths, not releases
or substitutes for owning GitHub issues. Sibling checkouts and prior artifacts
are preserved.
