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

Both packages inherit released workspace version 0.50.13 at
`650d98c1fe0ab144d5f4a83bab92e0ebd2c4f538`. Shared Tooling remains the reviewed
0.26.10 snapshot `a8141e775983c7fe504c4dd911e978152e657fdf`.

The incoming root lock update selects Host libraries 0.66.2 and Platform 0.6.11.
Their published Rust sources match the previous 0.66.0 and 0.6.10 selections.
Existing compatible requirements and prepared 0.66.0 release/Testkit CLIs remain
intentional; this update needs no helper rebuild. The independent runtime lock,
PocketIC 16.1.0 pairing, substrate 0.7.2 and compiler selections are unchanged.

Pending 0.50.14 adds bounded region snapshots, explicit collection constructor
contracts, deterministic recovery mutation campaigns and collection model tests.
Installed qualification now includes a separate Storage-owned fixture, preserving
Memory's dependency direction. The release helper separates metadata,
qualification and publication, bounds captured output and reports command time.
Fine-grained installer reporting belongs to
[Shared #303 — Report Cargo tool setup phases and durations](https://github.com/dragginzgame/shared-tooling/issues/303).
The 0.26.10 snapshot adds aggregate offline tool reports and read-only selection
JSON, literal formatter catalog admission and composed external IC dependency
checks. The current prepared tool selections retain their build identity.
The consumer release fixture uses Shared's companion-copy and parallel lint-order
helpers, with stage rosters and stubs still owned here. Adoption qualification
passes actual offline tool admission/reporting, formatter/hooks, declaration and
production-graph checks, release adapters and both interrupted bootstrap states.
Focused producer installer, formatter, selection, graph, boundary and Make probes
pass at the selected source. Logs remain at `/tmp/ic-persistence-shared02610/`.
Upstream hosted CI was still running at review; this is focused Linux acceptance.
No vendored implementation is patched for that follow-up.

Audit implementation qualification logs are retained at
`/tmp/ic-persistence-audit-improvements/`; focused qualification passes all-target
Clippy, Rust 1.91 all-target compilation,
183 Memory and 12 Storage library tests, the 10,000-mutation recovery campaign,
61 release-helper regressions, pin/graph/Storage-boundary checks and both actual
interrupted bootstrap states. Installed Memory/Storage checks pass original,
O3, Os and Oz at `target/qualification/runtime/attempt.mEWmdj/`. Earlier evidence
remains tied to its
original source in [the runtime notes](../runtime-io-qualification.md) and published
changelogs. No new release, publication or hosted matrix pass is implied.

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
