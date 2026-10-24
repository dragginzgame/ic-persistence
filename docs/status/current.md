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

Both packages inherit released workspace version 0.50.14 at
`ccb062dc7ad0d27d7112c1e115186a76275e1cc9`. Historical consolidation, tooling
adoption and installed qualification retain their source scopes in the published
changelogs and [runtime notes](../runtime-io-qualification.md).

Pending compatible 0.50.15 selects IC Tooling 0.67.0 in the root native catalog
and both helper CLIs, with only those five selected lock packages changing.
Platform remains 0.6.13 in the root lock and 0.6.8 in the independent runtime
lock. The PocketIC 16.1.0 pairing, substrate 0.7.2, feature boundaries and
compiler selections remain. The release adapter selects explicit configuration
budgets (1 MiB/file, 4 MiB total, 64 files, include depth 16); source hashes and
compiler override admission retain their existing contracts.

Shared revision `4c0f198e56d4c6d27b12a5c2d515e0e01070151e` is exported from a
clean detached source with 77 roots and 118 payloads. The committed version label
is still 0.26.10; the dirty sibling 0.26.11 version edit is excluded. Setup now
reports phase/outcome/duration on stderr, delivering
[Shared #303 — Report Cargo tool setup phases and durations](https://github.com/dragginzgame/shared-tooling/issues/303).
Common build identities and prepared tool reuse remain intact. Build
and Testkit 0.67.0 reused completed shared caches. The first Testkit wait timed
out after 61 seconds behind IC Jobs; its retry admitted the completed cache once
that owner exited. The failed attempt remains retained.

Focused qualification passes strict all-target Clippy, actual Rust 1.91
all-target compilation, all five configuration-admission tests (including four
budget failures), three selected metadata/setup-order tests and nine native
IO/accounting tests. Pins, production graph and declaration boundaries, release
adapters, both actual interrupted bootstrap states, hooks, failure evidence,
formatting and tooling lint pass. Installed Memory/Storage qualification passes
original, O3, Os and Oz at `target/qualification/runtime/attempt.gdjAtr/`.
All 36 Memory IO rows match the prior retained qualification log.

Qualification and setup logs are retained under
`/tmp/ic-persistence-shared-timing-tooling067/`. Upstream native hosted matrices
were still running at review; Tooling's optimizer, installed runtime and browser
workflows had passed at the selected source. Local qualification does not imply
a release, publication or native macOS acceptance.

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
