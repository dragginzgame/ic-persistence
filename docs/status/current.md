# Current handoff

The workspace contains the `ic-storage` governed collection adapter only.
IC Memory owns allocation, runtime bootstrap and exclusive leases; upstream
`ic-stable-structures` owns collection algorithms and persisted formats. IcyDB
owns application codecs and recovery. See [current ownership](../upstreams.md).

The engine and benchmark retirement is committed in `d4a4f6f`, followed by release
`0.5.2` at `009412d`. Their [archived research](../archive/private-btree/README.md)
and retained artifacts remain historical evidence, not active implementation
requirements. No reader converts retired custom-format bytes into upstream data.

The [root changelog](../../CHANGELOG.md) owns pending numbered notes. Package
versions, the Git index and release refs remain maintainer-owned. Shared Tooling
is recorded in the root snapshot. No sibling checkout is edited by this pass.

## Release preflight completion: pending 0.5.7

The checkout is released 0.5.6 at `a8a9c99`. Its incoming changelog explanation
is preserved. Authorized release preparation now installs the metadata editor
alongside common tools, then checks both. Standalone preflight remains offline
and check-only. The preflight binds the editor selector as well as the common
catalog against drift during setup. No release, version change or Git index/ref
mutation is part of this fix.

`make test-release-preflight` exercises the actual local adapter and shared
preflight dispatcher with substituted Git, Cargo and tool effects. Six cases
cover fresh setup, prepared offline admission, missing/stale editor refusal,
compiler-change repair and failed installation without metadata changes.
The focused gate joins `check-release-commands`. Native Linux fixture checks,
ShellCheck, snapshot integrity, pins, declaration boundaries and documentation
links pass. Full CI, a live release and native macOS remain unrun.

Platform's concurrent [PR #32](https://github.com/dragginzgame/ic-platform/pull/32)
now supplies the interface tracked by
[Platform #30](https://github.com/dragginzgame/ic-platform/issues/30).
An overlapping isolated draft was stopped in favor of the existing contribution.
Review identified missing codec exports: Storage currently exposes the full
upstream `storable` namespace, so adopting only `Bound`/`Storable` would remove
existing public codec names. A focused compatibility patch and original-type
assertions are prepared separately; producer integration/publication still
precedes ordinary Storage adoption. The actual library declarations, private
substrate alias and lock selection remain unchanged here.
The [compatibility review](https://github.com/dragginzgame/ic-platform/pull/32#issuecomment-6268357398)
contains the exact export completion. The actual PR plus that patch passes its
two native Rust 1.88 identity/consumer tests, strict selected Clippy and Rust 1.91
Wasm compilation. An import-only Storage 0.5.6 source copy against the actual PR
plus that patch passes all ten adapter tests and Wasm checking on Rust 1.91 with
published Memory 0.48.2. This is source qualification, not ordinary adoption.
Evidence and the patch remain under
`/tmp/ic-storage-next-improvements.BJsY3o/`.

[Issue #12](https://github.com/dragginzgame/ic-storage/issues/12) remains blocked
on a released discard primitive. Latest upstream remains 0.7.2; open upstream
[PR #440](https://github.com/dfinity/stable-structures/pull/440) at `3531464`
contains a genuine internal replacement that skips reading the displaced lazy
value, but does not supply discard removal. No returning-wrapper workaround,
fork, dependency update or performance claim is added. Consumer IC instruction,
cycle and Wasm measurements require a supplying accepted/released implementation.

The following adoption/qualification sections retain their original source-bound
history; their earlier pending states are not the current checkout status.

## Shared Tooling 0.25.0 adoption: pending 0.5.5

The root snapshot now selects committed source
`2a79f5f93383f3d2a8a38d65406c7d52c2f77913`, matching Shared Tooling's observed
remote main. The canonical exporter installs its 104-file closure and retires
the two superseded Perl Cargo editors. Common setup no longer selects the unused
Candid extractor; retained tool installations are preserved. Both dependency
catalog entries are production dependencies, including IC Memory's test usage;
there are no exclusively test/tooling catalog entries.

Grouped Cargo formatting and release ordering share one policy. Release candidate
rendering uses published IC Build 0.18.0, with explicit
`make install-release-tools` setup and an offline `make release-tools-check`.
The local adapter supplies the admitted absolute executable path and a 1 MiB
per-document input bound; Shared Tooling owns the preparation transaction.

The stricter external-IC dependency rule cannot yet be fully implemented:
[Platform #30](https://github.com/dragginzgame/ic-platform/issues/30) owns the
missing stable-structures interface. Existing direct substrate access is recorded
migration debt, not an approved exception or a claim of full policy compliance.
The workspace declaration boundary remains active. There are no test-only
catalog entries to exclude from this library's production graph, and this library
has no deployment entrypoint. Consumer deployments retain their own graph gates.

The incoming IC Memory 0.48.2 lockfile selection is preserved. This adoption
changes no library code, collection formats, package versions or Git index/refs.
The checkout remains based on 0.5.4 and behind the two already-observed main
commits; this pass does not integrate their compiler-selection change.

Focused Linux qualification on Rust 1.99.0 passes snapshot integrity, prepared
common/release tools, stable grouped formatting and unsorted-group refusal,
pins, license, declaration boundary, documentation links and release dispatch.
The actual released editor prepares and validates a disposable 0.5.5 candidate,
preserves external lock selections and refuses an undersized input allowance
without changing the fixture's files. Selected-commit checking independently
admits 0.5.4 while the working tree has pending notes. Shell syntax, ShellCheck
and diff checks pass. Evidence is retained at
`/tmp/ic-storage-shared-adoption.bNJJTv/`. No full suite, release execution,
installed-IC or native macOS qualification ran; no library behavior changed.
The following sections retain their original source-bound qualification history.

## Development compiler

The development selector and explicit Linux/macOS CI installation use Rust
1.99.1. The library's Rust 1.91 minimum and dependency lock remain unchanged.
The root changelog collects this compatible contributor-tooling update under
0.5.5; package versions and release refs remain maintainer-owned.

## Governed retained regions: published 0.5.4

`BTreeMapRegion` retains an existing shared runtime lease and opens the exact
upstream tree region for each operation. Bounded framing reads precede the
consumer's retained-data admission; namespace key scans return owned snapshots.
Initialization refuses nonempty regions, and reservation reports region-limit
or typed IC Memory refusal without changing retained bytes or accounting.
The [package guide](../../crates/ic-storage/README.md#retained-trees-inside-a-shared-allocation)
describes IcyDB's page-257 integration and application-owned framing, namespace
and capacity checks. The implementation addresses
[#15 — Support retained governed collection regions sharing one control allocation](https://github.com/dragginzgame/ic-storage/issues/15).

The three focused region tests pass natively and on Rust 1.91 with the locked
IC Memory 0.48.0 graph. They also pass in a disposable copy selecting published
IC Memory 0.48.2, without changing this workspace's lockfile. Tests seed the
existing upstream restricted tree, interleave namespace mutations, cold reopen,
retain adjacent frames, reject malformed framing/unknown namespaces, and verify
growth refusal, retry and exclusive-lease release. Strict package Clippy, Wasm
compilation, ten documentation examples, formatting, documentation links and
the dependency boundary pass. This is adapter qualification, not a live canister
upgrade or IcyDB adoption. Full release CI and native macOS remain unrun.

Release 0.5.3 is committed at `249da24992da229464b809d44ffed7f5ab8d4bae`.
The additive region interface shipped in 0.5.4 at `26a6390`. The following sections
retain the preceding 0.5.3 implementation and composition evidence.

The ownership cleanup removes obsolete private-engine import probes and a test
of IC Memory's standalone unknown-key policy. Adapter tests retain persisted
round trips and exclusive-lease retention, reservation, clearing and reset.
The dependency-pin check now passes in this checkout: the retired benchmark
manifest is no longer tracked, so no staging workaround is needed.

Focused native build, strict Clippy, warning-free Rustdoc, Wasm compilation,
Rust 1.91 tests and documentation examples pass. Snapshot, formatting, pin,
license, dependency-boundary, documentation-link and release-command checks pass.
Full release CI, clean-source package verification and macOS execution are not
run by this cleanup.

## Published 0.5.3 adapter completion

Release 0.5.3 incorporates the supported adapter work from PRs #2, #3 and
#10 against the post-retirement library. `Memory` and `Bound` expose the exact
upstream interfaces without granting allocation authority. `BoundedCell` enforces
declared encoding bounds before writes and retained-value decoding, while the
application retains record schemas and recovery policy. Map clearing uses the
single `clear` name; consumers of the proposed `clear_new` method must use it.

Issue #11 is addressed by consuming page reservation and `allocated_pages`:
both return the collection while retaining its sole allocation lease. The
reservation contract is an absolute virtual page extent, not guaranteed entry
capacity. Callers own and qualify byte/page budgets; upstream node-layout
constants are not a supported entry-reservation guarantee. Controlled backing
refusal proves unchanged bytes/accounting, retry, insertion and cold reopening.
Seven unit tests and six Rustdoc examples pass, including on Rust 1.91; strict
Clippy, warning-free Rustdoc and Wasm compilation pass. Full release CI and
native macOS execution were not qualified by that focused adapter pass.

Issue #12 still requires an upstream discard-value primitive. The selected
substrate's returning mutations materialize the old value; dropping it in a
new facade method would not eliminate that work. No fork, second map, or custom
tree is introduced as a workaround, and no performance gain is claimed.

## Fleet composition follow-through

These notes retain the preceding composition work; remote PR status and consumer
release readiness have not been requalified by this repository-only cleanup.

The adapter now supports consuming map reservation and vector reset, plus
in-place map clearing, while retaining the exclusive allocation lease. These
operations cover Canic's receipt preallocation and blob-payload reset needs
without exporting raw memory or managers. These operations shipped in 0.5.3.
Native tests, strict native
lint, Rust 1.91 tests, Rustdoc boundary examples and Wasm compilation pass.

The explicit runtime handoff is prepared in [Canic #595](https://github.com/dragginzgame/canic/pull/595)
and [IcyDB #376](https://github.com/dragginzgame/icydb/pull/376), with consumer
adoption in [IC Crates #7](https://github.com/dragginzgame/ic-crates/pull/7). The paired
upstream Memory 0.44 sources pass a disposable composition test using one shared
runtime identity. This is not qualification of IC Crates' released graph.
Canic's adapter migration still needs a committed base for its pending Memory
0.48 adoption and the supported adapter operations; its normal dirty checkout
is preserved. No raw collection bridge is claimed removed yet.
