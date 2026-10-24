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

Pending compatible 0.50.17 selects Platform 0.6.15 at
`6dcf54a229201f85775d218b7a9f41809297878f` in both maintained dependency catalogs.
The incoming root lock already selected it and is retained. The independent lock
updates only Platform 0.6.8 to 0.6.15, preserving Tokio 1.53.3, PocketIC 16.1.0,
all other package records and dependency edges. Stable-structures remains 0.7.2;
its Platform facade and the PocketIC facade are unchanged between those releases.
Wire/schema features remain unselected, so no source API adaptation is needed.

IC Tooling 0.68.0 at `8f281177faaeee0adc51edc1031d4a7b268b0ce0` remains the latest
published release, confirmed against the official registry. Its selected five
native dependencies and both helper CLIs are retained. The sibling's local
0.68.1 preparation still declares package version 0.68.0 and is not a new
published selection. Earlier 0.68 qualification remains recorded in
`/tmp/ic-persistence-tooling068/` and the numbered changelog.

Shared snapshot `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`, compiler pins,
effective feature scopes, public APIs, allocation custody and retained formats
remain unchanged. Strict all-target Clippy, all 195 library tests, actual Rust 1.91
all-target checks for both workspaces, pin/graph/declaration boundaries and actual
offline tool/server admission pass. Installed Memory/Storage IO, upgrades and
rollback pass original/O3/Os/Oz at `target/qualification/runtime/attempt.HQqpM2/`.
Formatting and documentation links pass. Logs remain at
`/tmp/ic-persistence-platform0615/`.

Fresh isolated RustSec revision `21d909e4c8bd4e14c7aea58bf980c881a0053ab1`
reports zero vulnerability-list matches for both maintained locks with explicit
`--no-fetch --no-yanked`; existing unmaintained warnings remain. This does not
qualify yanked-package status. Platform's separate all-feature crypto finding is
tracked by [#50 — Qualify the crypto-bigint advisory](https://github.com/dragginzgame/ic-platform/issues/50);
crypto-bigint is absent from these two selected graphs. The advisory workflow's
missing registry-index records are separately reported as
[#56 — Prepare registry index coverage before yanked-package audit checks](https://github.com/dragginzgame/ic-platform/issues/56).
Upstream native CI remains in progress and its advisory workflow is not passing;
local qualification, release/publication and native macOS acceptance retain their
separate scope.

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
