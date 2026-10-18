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
remains at the reviewed 0.23.0 snapshot. No sibling checkout is edited by this pass.

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
