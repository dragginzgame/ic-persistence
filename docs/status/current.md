# Current handoff

The released baseline is 0.51.3 at `6cb8082`. The pending compatible 0.51.4 batch
selects IC Tooling 0.68.7, Platform 0.6.18 and Shared Tooling 0.26.20. Package
versions, public library APIs, stable formats and release receipt JSON remain
unchanged. Existing staging remains user-owned and untouched.

## Current selection

Both catalogs select Platform 0.6.18 at
`98a2a8aa9b015c730ef222941b387674a729b0ab`. Its Rust changes add optional BLAKE3;
neither graph enables it. Consumed stable-memory/PocketIC source and the qualified
registry stable-structures 0.7.2 substrate remain unchanged. Root feature selection
is `stable-structures`; the independent runner uses `pocket-ic`/`principal`.

The incoming root lock already selects Platform 0.6.18, the five native Tooling
libraries 0.68.7 and Metrics 0.16.2; its bytes remain exact. Compatible minimums
and both release-editor/server CLI pins now select 0.68.7 at
`7ae301d00353e1e27e94300d492e47f9736941c3`. Tooling shares Cargo completion
tracking and preserves typed native error sources. Testkit shares final startup
deadline/server admission across synchronous and async construction. Consumed
metadata/release APIs, CLI arguments and receipt formats remain unchanged.

The earlier independent lock refresh selected 38 compatible versions, including Tokio 1.54.1,
Candid 0.10.39, ICU 2.4 and HTTP/TLS dependencies. PocketIC client/server remain
16.1.0. Both exact CLI generations were already complete in the shared store;
explicit setup reused them and the retained server without tool compilation or
foreign-lock removal. Common tool pins, compiler selections, default features
and shared installer policy identities remain unchanged.

Shared advances to `a7f85d45d4212d4c39225f58597580560a333550`: 77 roots, 120 payloads.
Its compact core derives the specialist engineering contracts, reporting metrics
are opt-in, and focused checks follow coherent edits. Selected executable scripts,
catalog pins and installer policy identities are unchanged. Its npm Platform
boundary adds no dependency to this Rust-only consumer. Platform 0.6.18 remains
the latest tagged release; sibling drafts are excluded.
Normal releases start fresh; recovery uses `release-resume VERSION=X.Y.Z`.
Consumer receipts, archives and failed logs retain their source custody; candidate
runner journals are not historical archives. See [pending notes](../changelog/0.51.md#0514).

## Qualification and limits

Current snapshot/pin, production graph, external dependency boundary, formatting
and documentation-link checks pass. All 62 release-helper tests pass against
the selected 0.68.7 native libraries. Both CLI installations reused verified
shared-store generations; offline release-editor and runtime-server admission pass.
The actual selected editor admits both interrupted metadata states in
`/tmp/ic-persistence-bootstrap.IsmwgF/`. Installed Memory/Storage IO, upgrade and
rollback pass original/O3/Os/Oz at `target/qualification/runtime/attempt.mleLw4/`.
Current preview, setup and focused qualification logs are retained at
`/tmp/ic-persistence-adopt-20261026/`. Initial sandbox failures (read-only tool
store and refused loopback bind) remain separately from successful retries.
No lock selection changed in this task.

Earlier 0.68.6 qualification covered both locked metadata graphs, strict
native/test-support and Wasm library/example Clippy, independent-runner Clippy,
Rust 1.91, product/runner tests, consumer release/bootstrap and archive admission.
Those results retain their original inputs and do not qualify the new Tooling
selection by themselves.

Earlier three-archive native/MSRV/Wasm consumption and integrity passed at
`target/package-qualification/attempt.wDxp9p/`. Installed Memory/Storage IO,
upgrade and rollback passed original/O3/Os/Oz at
`target/qualification/runtime/attempt.Oyk5BC/`. Earlier source comparisons,
registry observations, index/source preimage and focused qualification logs are at
`/tmp/ic-persistence-upstream-next/`. Full consumer release and hosted macOS
qualification remain separate; no commit, release or publication is performed.

Earlier [Tooling supplier CI](https://github.com/dragginzgame/ic-tooling/actions/runs/40295204797)
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
