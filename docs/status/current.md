# Current handoff

IC Persistence combines the Memory and Storage repositories. Memory owns
allocation governance, bootstrap and exclusive leases; Storage owns governed
collection adapters over those leases. Crate names and public responsibilities
are unchanged. See the [migration map](../migration/ic-persistence.md) for exact
source revisions, preserved branches/PRs, exceptions and cutover instructions.

The consolidated checkout is `/home/adam/projects/ic-persistence`, on `main`,
with origin `https://github.com/dragginzgame/ic-persistence`. Integration was
prepared on `integration/ic-persistence` in `/tmp/ic-persistence-integration`.
The import commit preserves Storage's original history as a second parent;
workspace/tooling edits are separate. The maintainer authorized
publication to the new public `dragginzgame/ic-persistence` repository on
2026-10-20. The destination exists with collaborator-only PR creation; the
consolidated branch was selected for its `main`. Downstream consumers retain their own dependency adoption decisions.

Both packages inherit released workspace version 0.50.11 at
`e17b50ae1288a4e89c4affaab5f7475440c99530`. Its complete hosted CI passed;
[Persistence #130](https://github.com/dragginzgame/ic-persistence/issues/130)
is delivered and closed. Historical consolidation, dependency and runtime
qualification retains its source/version scope in the numbered changelogs and
[runtime notes](../runtime-io-qualification.md).

Pending compatible 0.50.12 selects Host/Tooling 0.65.0 in the root catalog and
helper CLIs. The five Tooling versions/checksums advance from the incoming 0.64.2
lock selection; its Platform 0.6.9 remains. Both catalogs retain compatible
Platform 0.6.8 requirements, and the independent lock is unchanged. Consumed
Host library Rust sources and effective features are unchanged. The optional
Wire migration is outside this consumer. Compiler identities, substrate 0.7.2
and PocketIC client/server 16.1.0 remain selected.

Tooling's supplying source is `378ad71296db81436b38346b697960199bdb00e1`;
optimizer, installed runtime and browser workflows pass, with native CI still
pending at review. Focused local Clippy/MSRV, pin/graph, formatting, documentation
and lint checks pass, together with 60 release-tool tests, release/bootstrap
fixtures and nine native IO/accounting tests. Both 0.65.0 CLIs pass offline
admission. All four installed Wasm variants pass at
`target/qualification/runtime/attempt.Umus4m/`; the 36 IO/instruction rows match
the retained 0.64.1 log. Logs remain at `/tmp/ic-persistence-shared0267-host065/`.
These are focused Linux results, not full release or hosted consumer acceptance.

Shared 0.26.7 with merged [PR #302](https://github.com/dragginzgame/shared-tooling/pull/302)
is adopted at `35bc5a90e528ea2f60312997ea99a5969296248c` through the canonical
exporter (75 roots, 114 payloads). The maintainer authorized the upstream merge
and exact-source adoption. Explicit Cargo setup now waits up to 60 seconds on
admitted contention, including normal owner cleanup; offline checks stay
immediate. Existing tool identities and qualified binaries remain reusable.

The initial 0.26.7 producer run passed Linux/ARM macOS but exposed the Intel
owner-cleanup race fixed by #302. Deterministic withdrawal/replacement regression
and complete installer fixtures pass locally. The merged source also passes consumer setup/offline admission, snapshot,
release/bootstrap, hook and failure-evidence/retention fixtures. Its native matrix
remains pending, so #301 stays open. No distributed payload is patched by hand.

The delivered tooling requires inherited member manifests and matching local lock
versions before bootstrap. Archive qualification consumes the checked catalog
MSRV. Runtime CI cache keys bind tool selections and Shared's build identity.
CI retains its failure-evidence fixture for the hosted archive round trip;
normal packaging and archive-consumer qualification remain independent gates.

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
