# Current handoff

The released source baseline is 0.51.1 at `fa5cfd7`. The pending compatible
0.51.2 batch implements the reviewed complexity reductions. Public APIs,
receipt JSON fields, stored formats, package versions, Rust compiler selections
and the real Git index are preserved. The same batch adopts the dependencies
described below, retaining the separate workspace boundaries.

## Complexity reductions

The Rust release adapter and shell bootstrap/archive callers consume the same
ordered package roster. Expected source projections are reused within admission
phases while current files and identities remain reobserved around effects.
Receipt stage/archive location and fixed preparation steps are explicit. Native
status-only execution and bounded output capture have distinct results. Release
fixtures admit typed faults and separate process families without introducing
another runner or changing the retained receipt schema.

Local formatter modes share one operation list across both workspaces and the
standalone package consumer. The installed qualification script iterates Memory
and Storage artifacts through original/O3/Os/Oz, preserving selected catalog
inputs, optimizer flags, hashes/logs, Testkit lifecycle and Make build admission.
The external-declaration ceiling has one private owner. Fixture doctor status is
projected from the same cell read used for recovery; public DTOs and observations
before policy callbacks retain their existing contracts.

## Follow-up cleanup qualification

The subsequent pass implements nine further reductions: archive-map projections,
fully typed fault callers, named release inspections, fixture revision projection,
carried source/candidate bytes, one Wasm probe build/budget roster, locally callable
CI tool reuse, private historical-key lookup and named installed snapshots.
Histogram continuity keeps its bounded loop; a replacement would add projection
machinery without matched Wasm-cost evidence. No product state or stored format
is added.

This pass preserves the incoming root lock, which now selects Tooling 0.68.4;
the earlier 0.68.3 adoption evidence below retains its original selection scope.
CLI pins, both catalogs/lockfiles, compiler pins, the shared snapshot and the real
index are unchanged by this pass. Missing locked sources were fetched separately;
all compilation/qualification then ran offline.

Strict Memory all-target/test-support and independent-runner Clippy pass, as do
62 release-helper, 62 runtime, 47 ledger, seven request and two runner tests.
Both affected compiler-floor checks, archive provenance/refusal fixtures,
release/parallel-Make/bootstrap fixtures, offline tool reuse, formatting, tooling
lint and dependency/snapshot boundaries pass. All four Wasm probes remain within
their budgets. Actual archive/native/MSRV/Wasm consumption and integrity pass at
`target/package-qualification/attempt.peGojm/`; installed Memory/Storage IO,
upgrade and rollback pass original/O3/Os/Oz at
`target/qualification/runtime/attempt.iYaO7a/`. Logs and the incoming preimage are
retained at `/tmp/ic-persistence-complexity-next/`. Full release and hosted
qualification remain separate; no release or Git delivery is performed.

## Dependency adoption

The incoming root lock selects published IC Tooling 0.68.3, Platform 0.6.16 and
Metrics 0.16.2. Root requirements and both helper CLI selections are aligned;
the root lock bytes are preserved. The independent runtime lock changes only
Platform 0.6.15 to 0.6.16 and its checksum. PocketIC client/server 16.1.0,
stable-structures 0.7.2, Tokio 1.53.3 and all other selected packages/edges are
unchanged. Optional Wire support remains off in the consumed Tooling libraries.

Published sources are Tooling `600fbd0db3ad761a92cf8418daac3517539b1da2`,
Platform `0b84c2eb68239aaee4a95de35f82e702e138dcf0` and Metrics
`0d60020987fe706e8182b6d4ab9c87a406876f16`. Metrics' Rust sources match 0.16.1;
Platform's stable-memory and PocketIC facades are unchanged. Build metadata and
release rendering, metered memory ownership and Testkit startup refactors are
qualified through this consumer's actual selections. Both new CLIs reuse
admitted installations without compilation; existing server bytes are retained.

Full locked metadata and policy checks cover both graphs. Native, Wasm and
independent-runtime Clippy and actual Rust 1.91 all-target compilation pass.
Installed Memory/Storage IO, upgrades and rollback pass original/O3/Os/Oz at
`target/qualification/runtime/attempt.vC8c3b/`. All 62 release-helper, 62 Memory runtime, 12 Storage and 27 Metrics Stable
tests pass, along with their selected doctests and release/archive-admission
fixtures. Actual three-archive qualification passes at
`target/package-qualification/attempt.yaM5oc/`, including native/MSRV execution,
Wasm compilation and final integrity checks. Source comparisons and logs remain
at `/tmp/ic-persistence-deps0683/`.
No product Rust code or prior cleanup implementation is changed by this adoption.

Metrics' supplier native/MSRV matrix passes. Platform's native matrix and runtime
job pass; its advisory job retains the crypto-bigint finding tracked by
[Platform #50 — Qualify the crypto-bigint advisory](https://github.com/dragginzgame/ic-platform/issues/50)
and the missing index coverage tracked by
[Platform #56 — Prepare registry index coverage before yanked-package audit checks](https://github.com/dragginzgame/ic-platform/issues/56).
Crypto-bigint is absent from both selected Persistence graphs. This is graph
reachability evidence, not a fresh advisory audit or yanked-package qualification.

Tooling's Linux/macOS ARM native, MSRV, optimizer, session and PocketIC jobs pass.
Its macOS Intel native job fails on cancellation-fixture readiness (empty marker
bytes versus `armed`), tracked by
[Tooling #251 — Qualify native cancellation readiness on macOS Intel](https://github.com/dragginzgame/ic-tooling/issues/251).
That source failure is not reported as a passing native matrix. The affected
Build native wrapper is not called by this consumer; own hosted macOS and full
release qualification remain separate. Unpublished Tooling 0.68.4 work is excluded.

## Earlier complexity qualification

The following evidence predates the dependency adoption and retains its original
selection scope.


Strict Memory all-target Clippy with test support, all 62 release-helper tests,
62 focused runtime tests, seven declaration tests and the actual Rust 1.91
all-target check pass. Compiler/archive admission, malformed/duplicate/escaping
roster refusal, release/parallel-Make/bootstrap fixtures, actual Git hooks,
formatting, tooling lint and dependency/snapshot boundaries pass.
Actual three-archive qualification passes at
`target/package-qualification/attempt.2F9RbZ/`, including native/MSRV consumer
execution, Wasm compilation and final archive/unpacked/source integrity checks.
Installed IO, upgrades, rollback and Storage collection checks pass all four
variants at `target/qualification/runtime/attempt.OaFKDJ/`. The sandboxed attempt
`attempt.vGXwqF` remains retained: its server could not bind loopback before any
qualification; native execution used the same prepared server/tool selections.
Logs and task preimages remain at `/tmp/ic-persistence-complexity/`.

[Shared Tooling #308 — Diagnose unsupported EXIT-trap shapes in fixture_probe_prefix](https://github.com/dragginzgame/shared-tooling/issues/308)
tracks the reproduced silent refusal of a multiline trap by the generic retention
probe. No shared payload or sibling checkout is patched. The shared snapshot
remains 0.26.16 at `2077c934efb82b181bea58fe50078dbad289c1aa`.
The complete release gate, hosted macOS qualification and release/publication
remain separate. This work stages, commits, pushes and releases nothing.

## Repository retirement

All 121 original issues are now in IC Persistence, preserving their open/closed
states. All 362 original issue comments were verified: 341 transferred normally;
21 omitted from destination discussions were restored with explicit original
authorship and timestamps. The [issue](../history/issue-transfers.json) and
[comment](../history/comment-transfers.json) maps retain old-to-new references.
All 23 original PRs have [historical exports](../history/pull-requests/README.md)
with descriptions, comments, reviews, timelines, commits and diffs.

The maintainer authorized deletion of both old local checkouts and both original
GitHub repositories. The local checkouts were deleted after verified backups.
Complete checkout archives, verified Git mirrors and GitHub exports remain at
`/home/adam/projects/ic-persistence-retirement-2026-10-20/`.
Both original GitHub repositories remain: deletion was refused because the
current CLI token lacks `delete_repo`. Both old trackers report no remaining
issues; the nine source copies flagged during approval review were independently
verified, including all 21 associated comments, before retrying deletion.

## Historical consolidation qualification

The original integration's archive/MSRV/Wasm, source-preservation and bootstrap
proofs retain their source scope in
[the release-time handoff](https://github.com/dragginzgame/ic-persistence/blob/4dda4c1511f09f6bd14d1ea8ae8d428dccdd2f75/docs/status/current.md#historical-consolidation-qualification).
Evidence remains under `/tmp/ic-persistence-preservation/` and the integration
checkout's `target/package-qualification/`; original plans and checkout artifacts
remain in the verified retirement backups above. Preserve failed attempts and
input hashes. These historical results do not qualify later changes.
