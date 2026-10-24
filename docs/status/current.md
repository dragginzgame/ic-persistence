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

Both packages inherit released workspace version 0.50.15 at
`7deafe9c42926c8ecd01952277b571813c21a04f`. IC Tooling 0.67.0 and Shared
revision `4c0f198e56d4c6d27b12a5c2d515e0e01070151e` remain selected.
Published changelogs retain earlier adoption/qualification scope.

Pending compatible 0.50.16 updates only Tokio 1.53.2 to 1.53.3 in the independent
runtime lock. All other dependency identities and edges, the root library lock,
PocketIC 16.1.0, both Platform selections, substrate and compiler pins remain.
A fresh isolated RustSec database at `21d909e4c8bd4e14c7aea58bf980c881a0053ab1`
reports the AsyncFd advisory before the patch and no longer reports it afterward.
The selected Linux dependency source scan found no `AsyncFd` references outside
Tokio; this is source reachability evidence, not proof of complete exploitability
or security. Existing unrelated unmaintained warnings are retained separately.
See [#132 — Qualify Tokio AsyncFd I/O safety](https://github.com/dragginzgame/ic-persistence/issues/132).

The map insertion review retains caller-owned key copies where post-insert
metadata needs the original key and returned previous value. The current
Platform boundary exposes no single-pass key-retaining hook; adding a copy
wrapper would not resolve that cost. The owned-key contract is documented without
a new API, engine or format. See
[#131 — Evaluate caller key access across returning insertion](https://github.com/dragginzgame/ic-persistence/issues/131).

Qualification evidence is retained at `/tmp/ic-persistence-issues131132/`.
Installed Memory/Storage IO, upgrade and rollback checks pass all four original,
O3, Os and Oz Wasm variants at `target/qualification/runtime/attempt.0OtT2b/`.
The independent graph also passes all-target compilation on Rust 1.91.0.
The dependency change is still a working-tree selection; release/publication and
native macOS acceptance retain their own authority.

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
