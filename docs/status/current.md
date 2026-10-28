# Current handoff

Released source is 0.52.1 at `406364c7a8651b23400ed8b10cb16030c333be2a`.
The working tree contains the maintainer-selected boundary-based cost correction
for [#137 — Fail closed when multiple memory runtimes alias one stable backing](https://github.com/dragginzgame/ic-persistence/issues/137).
The issue was reopened after IcyDB reported query-budget regressions. Original
0.52.0 custody delivery passed [all 10 CI jobs](https://github.com/dragginzgame/ic-persistence/actions/runs/40677730611);
that evidence remains valid for its original source, not this pending change.

## Accepted contract and pending 0.53.0

The maintainer explicitly selected trusting exclusive custody between checked
boundaries. Construction, initial bootstrap, successful new leases, diagnostics
and growth still validate full manager metadata. Growth verifies before mutation
and after projecting its expected edits. Size and safe/raw IO check poisoning
and virtual bounds without a metadata scan. Detected divergence permanently
poisons all handles; restoring bytes does not rehabilitate the owner.

Duplicate-owner refusal before backing IO, detached-handle custody and truthful
physical identities remain enforced. Hosts must retain one Memory crate identity
and prohibit raw aliases/unmanaged managers, including over excluded IDs. Raw
changes during a live lease need not be detected until a full checkpoint. No
alternate unchecked mode, unsafe optimization or persistent format is added.
Warm host-binding verification remains IO-free. See [pending notes](../changelog/0.53.md).
The draft is 0.53.0 because it narrows the advertised per-access detection
contract. Actual package versions remain 0.52.1; toolchains and the real index
remain unchanged. All edits are uncommitted. The runtime correction's source
bytes are preserved through the separate dependency adoption below.

## Matched investigation and qualification

Evidence is retained at `/tmp/ic-persistence-cost-137/`, including source/lock
hashes, the isolated IcyDB source/diff, controls, setup failures and Wasm artifacts.
No sibling checkout is modified. The IcyDB snapshot is source
`684fcc8bfabf6c2a713594e5f0d6ae41866a5c33` plus its captured incoming dependency
and bounded-loader edits. Query sources and budgets stay fixed. A local Memory
path override changes only verification placement between matched runs.

Both original grouping budget failures reproduce. With the correction, all nine
grouping tests pass; measured queries use about 132–160M instructions per call
inside their unchanged 4.5B allocation. The full 10,001-row collection case also
passes fact queries, bounded jobs, upgrade and recovery. Its preexisting bounded
256-row loader correction belongs to IcyDB and is preserved in the isolated copy.

For 1000 one-byte reads, Rust 1.91 raw-Wasm instructions fall from 192,300,274
to 393,274; actual complete-update charged cycles fall from 199,897,192 to
7,798,325. Cycles include update/open costs outside the inner instruction counter.
The temporary no-verification attribution control is not a candidate or proof.
Four-word comparisons and uninitialized-read experiments were not adopted.

69 Memory runtime tests, 16 bounded fixture tests, 17 Storage adapter tests and
six actual Rust 1.91 custody cases pass. Strict native Memory/example Clippy,
actual Wasm library/fixture Clippy and independent runner/test Clippy pass.
Final original and Binaryen Oz installed Wasm pass bounds, rollback,
custody/composition and six open/growth metadata faults, including same-count
divergence. Fresh hosted/macOS CI and the full local
release gate have not run; delivery and required native acceptance remain open.

Rust 1.99.1 raw probes remain within existing budgets: core 249194/260000,
diagnostics 259994/315000, admission 253039/264000 and runtime 256238/270000.
They grow by 248–255 bytes against the matched unchanged source. IcyDB's raw
and canonical optimized grouping artifacts shrink by 1608 and 770 bytes.

## Dependency adoption and preserved history

The current task adopts Platform 0.7.3 in both catalogs and locks, native Tooling
0.69.6 and separate Build/Testkit 0.69.6 CLIs. The incoming library lock already
selected the new versions and is preserved byte-for-byte; only Platform changes
in the independent lock. Shared remains current at 0.27.3. Source review finds
no selected Platform interface or substrate/client/server changes. Dirty Platform
0.7.4 sibling work is excluded. Incoming diffs, locks, preserved runtime-source
hashes and focused adoption logs are at `/tmp/ic-persistence-adoption-073-0696/`.
65 helper tests, strict helper Clippy, all three Wasm library checks, Rust 1.91
helper/independent-workspace checks and selected-editor bootstrap pass. Both
0.69.6 CLIs are prepared and admitted offline; snapshot, pin/graph/boundary,
formatting and local-link checks pass. No new installed or macOS matrix is claimed.
Earlier cost/consumer measurements above retain their original dependency graph;
they are not relabelled as installed acceptance of this newer selection.

Released 0.52.1 selects Platform 0.7.2, native Tooling 0.69.5 and separate
Build/Testkit 0.69.5 CLIs. Shared 0.27.3 remains reviewed at
`7ee5c83744aa822ecd070703dbf0a7d95076d5f6`; stable-structures remains 0.7.2 and
PocketIC client/server 16.1.0. Earlier adoption evidence is at
`/tmp/ic-persistence-adoption-20261028/`; supplier and fixture evidence remain at
`/tmp/ic-persistence-issue-137/` and `/tmp/ic-persistence-fixture-custody-fix/`.
Earlier dependency observations remain at `/tmp/ic-persistence-deps-20261028/`.

Preserve all release journals, receipts, failed attempts and retained archives.
Original Memory/Storage history survives through the
[migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). Keep verified backups at
`/home/adam/projects/ic-persistence-retirement-2026-10-20/` and all mirrors,
exports and recovery plans. Original checkout retirement was authorized; last
GitHub deletions were refused for missing `delete_repo`, with no retry in scope.
