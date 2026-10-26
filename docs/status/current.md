# Current handoff

The released baseline is 0.51.3 at `6cb8082`. The pending compatible 0.51.4 batch
selects IC Tooling 0.68.6, Platform 0.6.18 and Shared Tooling 0.26.19. Package
versions, public library APIs, stable formats and release receipt JSON remain
unchanged. Existing staging remains user-owned and untouched.

## Current selection

Both catalogs select Platform 0.6.18 at
`98a2a8aa9b015c730ef222941b387674a729b0ab`. Its Rust changes add optional BLAKE3;
neither graph enables it. Consumed stable-memory/PocketIC source and the qualified
registry stable-structures 0.7.2 substrate remain unchanged. Root feature selection
is `stable-structures`; the independent runner uses `pocket-ic`/`principal`.

The incoming root lock already selects Platform 0.6.18, the five native Tooling
libraries 0.68.6 and Metrics 0.16.2; its bytes remain exact. Compatible minimums
and both release-editor/server CLI pins now select 0.68.6 at
`dede95dd1d6ea1cb79367ecd3957233fcf3a2c81`. Build strengthens Cargo completion and
diagnostic-read admission. Its consumed metadata/release APIs remain unchanged;
new Testkit benchmark helpers require no Persistence source migration.

The independent lock refreshes 38 compatible versions, including Tokio 1.54.1,
Candid 0.10.39, ICU 2.4 and HTTP/TLS dependencies. PocketIC client/server remain
16.1.0. Both exact CLI generations were already complete in the shared store;
explicit setup reused them and the retained server without tool compilation or
foreign-lock removal. Common tool pins, compiler selections, default features
and shared installer policy identities remain unchanged.

Shared stays at `c830c33447f0a84e015bda7fa14b6c096a58368e`: 77 roots, 119 payloads.
Normal releases start fresh; recovery uses `release-resume VERSION=X.Y.Z`.
Consumer receipts, archives and failed logs retain their source custody; candidate
runner journals are not historical archives. See [pending notes](../changelog/0.51.md#0514).

## Qualification and limits

Both locked metadata graphs, strict native/test-support and Wasm library/example
Clippy, independent-runner Clippy and actual Rust 1.91 checks pass. All 62 release-
helper, 183 Memory, 12 Storage, 27 Metrics Stable and three runner tests pass; seven
opt-in measurements remain ignored. Consumer release/bootstrap and compiler/archive
admission fixtures pass, as do formatting, lint and graph/snapshot boundaries.

Actual three-archive native/MSRV/Wasm consumption and integrity pass at
`target/package-qualification/attempt.wDxp9p/`. Installed Memory/Storage IO,
upgrade and rollback pass original/O3/Os/Oz at
`target/qualification/runtime/attempt.Oyk5BC/`. Current source comparisons,
registry observations, index/source preimage and focused qualification logs are at
`/tmp/ic-persistence-upstream-next/`. Full consumer release and hosted macOS
qualification remain separate; no commit, release or publication is performed.

[Tooling supplier CI](https://github.com/dragginzgame/ic-tooling/actions/runs/40295204797)
and its PocketIC/optimizer/session workflows pass all 14 checks on the three
supported hosts. [Platform supplier CI](https://github.com/dragginzgame/ic-platform/actions/runs/40308135437)
passes native, browser and installed runtime checks. Its advisory job still fails
on incomplete index coverage ([#56](https://github.com/dragginzgame/ic-platform/issues/56))
and the Agent graph's crypto-bigint finding ([#50](https://github.com/dragginzgame/ic-platform/issues/50)).
Neither consumer metadata graph selects crypto-bigint; this is not a fresh audit
of both consumer graphs or proof of general vulnerability absence.

Earlier adoption evidence remains at `/tmp/ic-persistence-deps0685/` and
`/tmp/ic-persistence-platform-shared/`, including archives `attempt.YMjCCf` and
`attempt.DG6fHT`, and installed `attempt.EhPPSl` and `attempt.KSDCQT`. Those
results retain their older Tooling/Platform selections and runtime lock scope.

## Retained evidence and recovery

The [committed 0.51.3 handoff](https://github.com/dragginzgame/ic-persistence/blob/6cb80827170277acc67bff847b9170a208a8e8e5/docs/status/current.md)
preserves earlier cleanup/dependency identities, qualification paths and supplier
uncertainties. Its pending-work descriptions are historical. Prior archives,
receipts and failed attempts remain retained with their original source scope.
The [ownership guide](../ownership.md), [host support](../host-support.md) and
[adoption boundaries](../shared-tooling-adoption.md) remain the active contracts.

All 121 original Memory/Storage issues, 362 comments and 23 PR histories were
preserved through the [migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). The maintainer authorized
retiring both original checkouts and repositories. The old local checkouts were
removed after verified backups at
`/home/adam/projects/ic-persistence-retirement-2026-10-20/`. Last recorded GitHub
deletions were refused because the CLI token lacked `delete_repo`; no deletion
retry is part of this work. Preserve the backups, mirrors, exports and recovery
plans. The [original consolidation evidence](https://github.com/dragginzgame/ic-persistence/blob/4dda4c1511f09f6bd14d1ea8ae8d428dccdd2f75/docs/status/current.md#historical-consolidation-qualification)
retains its original source scope.
