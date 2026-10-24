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

Both packages inherit released workspace version 0.50.16 at
`5856e8dfc431f92cc0e6159e4fda0b2a8074f810`. All ten
[delivered CI jobs](https://github.com/dragginzgame/ic-persistence/actions/runs/40086649292)
passed, including library/MSRV and installed runtime checks on all three native
hosts. The delivered Tokio 1.53.3 lock matches the earlier local audit and runtime
qualification for [#132 — Qualify Tokio AsyncFd I/O safety](https://github.com/dragginzgame/ic-persistence/issues/132).
Earlier source/evidence scopes remain in published changelogs.

Pending compatible 0.50.17 selects IC Tooling 0.68.0 in the root native catalog
and both helper CLIs at `8f281177faaeee0adc51edc1031d4a7b268b0ce0`. The five
consumed libraries' published Rust sources match 0.67.0. The optional Testkit Wire
feature owns the upstream breaking change and remains unselected here. The
producer's dirty 0.68.1 work is excluded; no source API adaptation is needed.

Only five native tooling versions/checksums change in the root lock. Platform
remains 0.6.14 in the root and 0.6.8 in the independent runtime lock, whose bytes
are unchanged. Tokio 1.53.3, PocketIC 16.1.0 pairing, substrate 0.7.2, compiler
pins, feature selections, public APIs and retained formats remain selected.
Shared snapshot `4c0f198e56d4c6d27b12a5c2d515e0e01070151e` is unchanged.

Both new CLI selections are prepared and pass offline admission using the existing
server and shared caches. Strict all-target Clippy, actual Rust 1.91 all-target
compilation, nine native IO/accounting cases and three selected metadata/setup
admission tests pass. Pin/production/declaration checks, both actual interrupted
bootstrap states, release adapters, hooks, evidence retention, formatting and
tooling lint pass. Installed Memory/Storage IO, upgrade and rollback checks pass
original, O3, Os and Oz at `target/qualification/runtime/attempt.2FpNKI/`.

Focused qualification logs are retained under
`/tmp/ic-persistence-tooling068/`. Producer optimizer, browser and runtime
workflows pass at the selected source; its native hosted matrix was still
running at review. Release/publication and native macOS acceptance retain their
separate authority.

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
