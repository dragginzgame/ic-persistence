# Current handoff

IC Persistence combines the Memory and Storage repositories. Memory owns
allocation governance, bootstrap and exclusive leases; Storage owns governed
collection adapters over those leases. Crate names and public responsibilities
are unchanged. See the [migration map](../migration/ic-persistence.md) for exact
source revisions, preserved branches/PRs, exceptions and cutover instructions.

The consolidated default branch released both packages at 0.50.1 in commit
`d61e5c014e11313313da167df3f24af2de6d96aa`. Both inherit the workspace version;
the old explicit Storage version and current-source pin exception are retired.
The release adapter still retains historical source projection and interrupted
metadata bootstrap for existing receipts. Preserve those receipts and artifacts.
The root snapshot selects committed Shared Tooling 0.25.9, including Storage
PR #18's release retry protection. Dirty Shared sibling work is not adopted.

Storage's original handoff and changelog remain under
[historical evidence](../history/ic-storage/handoff.md). Platform #30/#32 and
the former Storage #12 (now Persistence #118) remain upstream/consumer work.
Memory PR #125/#126 heads are retained for review; main already contains their
capabilities through later releases.

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

## Qualification and retained evidence

The initial Linux integration, archive/MSRV/Wasm consumers, release adapters and
bootstrap qualification retain their original source scope in
[the release-time handoff](https://github.com/dragginzgame/ic-persistence/blob/f452b5cf9b26a25ee2fb69f4941c079e9c85f4e7/docs/status/current.md#qualification).
Evidence remains under `/tmp/ic-persistence-preservation/` and the integration
checkout's `target/package-qualification/`. Preserve failed attempts and input
hashes; these results do not qualify later commits or native macOS.

The first hosted consolidated run exposed missing Rust 1.91 setup in the native
archive-consumer jobs. Separate installed-runtime and MSRV jobs passed on all
three hosts; the complete native gate is not yet qualified by that run.
See [run 39461398998](https://github.com/dragginzgame/ic-persistence/actions/runs/39461398998).
No tool/server refresh, deployed-canister reset or evidence cleanup belongs to
this policy cleanup. Historical original release plans and checkout artifacts
remain in the verified retirement backups above.
