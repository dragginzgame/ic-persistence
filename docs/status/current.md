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
consolidated branch was selected for its `main`. Package versions and downstream
consumers remain unchanged by this local setup.

Current package versions remain Memory 0.49.4 and Storage 0.5.9. The proposed
coordinated release is 0.50.0. The temporary explicit Storage version disappears
through authorized release preparation, not during repository consolidation.
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

## Qualification

The archive helper now seeds fixture locks from the repository selection,
rejects external version/source/checksum drift and duplicate or substituted
package identities, and builds with `--locked`. Native, Rust 1.91 and Wasm
archive-consumer checks pass, as do the injected drift/duplicate rejection
fixtures and final archive, unpacked-file and source-input integrity checks.

Linux: the focused workspace run passed 267 tests (including 60 release tests)
and 13 doctests; six existing allocation-measurement tests remain intentionally
ignored. Strict all-target Clippy and the Rust 1.91 all-target check passed, as
did the Wasm test build and production graph exclusions. The final publication
change was rechecked with the 12 publication/failure cases.

Both archives build independently. The extracted-archive consumer passes native
selected-compiler and Rust 1.91 execution, plus Wasm compilation, with one Memory
identity, exclusive leases and retained collection values. The independent runtime
runner builds with its original selected lockfile. Real offline Cargo/editor
checks admit all three interrupted two-package bootstrap states without input
mutation. Product sources, current fixtures and Storage's README/license remain
byte-identical to the inventoried releases.

Evidence is retained under `/tmp/ic-persistence-preservation/` and the integration
checkout's `target/package-qualification/`; bootstrap fixtures retain their own
input hashes and command output. Hosted CI, native macOS, installed PocketIC
execution and real registry publication are not claimed by these checks. The
existing installed-runtime CI lanes remain configured; no server/tool refresh
or deployed-canister reset is part of this migration.

Snapshot integrity, dependency inheritance/pins, licenses, current/archive links,
formatting, action/shell lint, fixture/evidence retention, index hook selection
and release adapter dispatch checks pass. Strict rustdoc passes for both packages.
The formatter and bootstrap fixtures cover the added archive-consumer source.

The independent runtime runner also passes Rust 1.91. Memory's four raw Wasm
probes pass the existing budgets: core 238,134 bytes; diagnostics 247,808; admission
241,968; runtime integration 244,157. These are current measurements, not a claim
of a measured size improvement. All 98 original saved release plans are complete;
no original release-state lock is active. Those plans remain in their old checkout.
