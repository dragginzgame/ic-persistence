# Current handoff

Released source is 0.52.0 at `3acc1a76f7aa59309aff718f2b4f2efbec0d12dd`.
[Its CI](https://github.com/dragginzgame/ic-persistence/actions/runs/40677730611)
passed all 10 jobs: tooling, native validation, Rust 1.91 MSRV and installed
original/optimized Wasm qualification across Ubuntu and both macOS architectures.
[#137 — Fail closed when multiple memory runtimes alias one stable backing](https://github.com/dragginzgame/ic-persistence/issues/137)
is closed with that delivered source and acceptance evidence. Earlier formatter
and singleton-loader issues #135/#136 were already closed. These runs do not
qualify the pending adoption below.

## Pending compatible 0.52.1 adoption

The working tree adopts Platform 0.7.2 in both dependency catalogs and lockfiles,
Tooling 0.69.5 in native dependencies, and separately selected Build/Testkit
0.69.5 CLIs. The incoming library lock already selected Platform 0.7.2 and
Tooling 0.69.5; its bytes are preserved. Only Platform changes in the independent
lock. Optional features, stable-structures 0.7.2, PocketIC 16.1.0 and toolchains
remain selected. Package versions stay 0.52.0; the index remains untouched.
See [pending notes](../changelog/0.52.md#0521---unreleased).

Shared Tooling 0.27.3 is exported from clean detached source at
`7ee5c83744aa822ecd070703dbf0a7d95076d5f6`, retaining 77 roots and 123 payloads.
The upstream commit was independently observed through GitHub. Adoption compares
payload bytes/modes against that exact source as well as local snapshot hashes.
Consumer hook fixtures use the canonical companion copier and stage its complete
regular-file closure. Optional sccache stays inactive; no scheduled maintenance
is activated or reconfigured. No dirty sibling source is adopted or modified.

Evidence and incoming lock copies are retained at
`/tmp/ic-persistence-adoption-20261028/`, including impact/refresh/source comparison,
locked cache preparation, selected CLI setup/admission and focused qualification.
Both 0.69.5 CLI selections reuse admitted builds, and server setup admits the
existing Testkit-selected PocketIC 16.1.0 bundle without deleting old selections.
The snapshot, pin/inheritance, production graph and declaration checks pass,
as do the real hook fixture, substituted release-dispatch fixture, formatting,
and 65 native repo-tool tests. Strict helper Clippy, all three Wasm libraries,
Rust 1.91 helper/independent-workspace checks and the actual selected-editor
interrupted-metadata bootstrap fixture also pass. ShellCheck and local links pass.
No full local release gate, new-source hosted CI,
release, publication or deployment ran for this adoption. Linux source/setup
checks do not qualify fresh native macOS behavior.

## Released backing-custody contract and evidence

Memory requires one truthful, IO-free physical backing identity across aliases
and one Memory crate identity per composition. Standard VectorMemory and default
canister backings identify aliases automatically. Claims reject duplicate owners
before backing IO and survive runtime drop through the final handle/collection.
Hosts share one admitted, committed runtime; raw managers, including over excluded
IDs, remain prohibited. Metadata checks compare the exact expected MGR V1 image
before cached translations, poison on divergence and never refresh/repair it.

Each valid size/IO check reads 34,848 metadata bytes; successful growth checks
before/after. This remains a material stable-IO cost, not a speed-up over the
older unchecked path. Checks are point observations, not atomic isolation
against raw writes, dishonest identity, other processes or separate crate copies.
See [released notes](../changelog/0.52.md) and the committed supplier implementation.

Earlier source/lock/Wasm hashes, focused native/MSRV/installed composition proof
and original/final measurements remain at `/tmp/ic-persistence-issue-137/`.
The retained-cell fixture repair is committed in `616f704`; all 16 fixture cases,
strict test-support lint and Wasm checks passed. Its logs and a copy of the first
failed release gate remain at `/tmp/ic-persistence-fixture-custody-fix/`.
Earlier dependency evidence remains at `/tmp/ic-persistence-deps-20261028/`.
Preserve those observations and all release recovery state with their identities.

## Retained history and recovery

All original Memory/Storage issues, comments and PR histories remain preserved
through the [migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). Preserve verified backups
at `/home/adam/projects/ic-persistence-retirement-2026-10-20/` and all mirrors,
exports and recovery plans. Original checkouts were retired with maintainer
authorization. Last recorded GitHub deletions were refused because the CLI token
lacked `delete_repo`; no retry is part of this work. Earlier package/runtime
archives, receipts, failed attempts and logs retain their original custody.
