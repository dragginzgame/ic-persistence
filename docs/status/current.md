# Current handoff

The released baseline is 0.51.0 at
`e3939cd`, with Memory, Storage and Metrics Stable sharing the release identity.
All ten [release CI jobs](https://github.com/dragginzgame/ic-persistence/actions/runs/40106916102)
pass, including native/MSRV and installed runtime checks on Linux and both macOS
architectures. Metrics uses the independent 0.16 arithmetic line; consumers must
match that public type identity. Earlier source/import/recovery evidence remains
in [the release-time handoff](https://github.com/dragginzgame/ic-persistence/blob/e3939cd/docs/status/current.md)
and the published changelogs.

## Pending 0.51.1 review

The incoming root lock selects registry IC Metrics 0.16.1 at
`b6193cb18b137f1764e9bcf5dbbb5c6f2329cd81`. Its 15 published Rust files match
0.16.0 and it retains no production dependencies. The root requirement remains
compatible `0.16`; its incoming lockfile bytes and the independent runtime lock
are preserved. Public APIs, V1 metrics records, allocation custody, collection
formats, Platform 0.6.15, Tooling 0.68.1, PocketIC client/server 16.1.0 and compiler
pins remain selected. No runtime code change was warranted by the paths reviewed.

Reviewed Shared Tooling 0.26.16 at
`2077c934efb82b181bea58fe50078dbad289c1aa` replaces the earlier snapshot through
the canonical clean-source exporter after impact review. The 77 roots now select
119 payloads. Actual tool and server checks reuse admitted installations.
Shared's [portable CI](https://github.com/dragginzgame/shared-tooling/actions/runs/40171556775)
passes on all three hosts; this consumer's new working files have local focused
qualification, separately from that supplier evidence.

Runtime-server admission preserves busy status 75 without recommending setup;
ready, busy and missing CLI/server selections are exercised through the actual
Make target with substituted tool effects. Exact-archive consumer qualification
also passes at `target/package-qualification/attempt.ozDTjn/`: all three archives
compile, the consumer runs on Rust 1.99.1 and 1.91.0 and checks for Wasm, and final
archive/unpacked/source hashes match. Archive-admission fixture failures
replay retained diagnostics and preserve the original status. The explicit
failure probe retains status 29 and the original packaging diagnostic.

Full locked metadata for both workspaces, pin/graph/declaration policies, actual
offline tool/server admission, native and Wasm adapter Clippy, all 27 adapter
regression tests, three development and three MSRV doctests, package-admission
and release/bootstrap fixtures, Git hook behavior, formatting and tooling lint
pass. Evidence remains at `/tmp/ic-persistence-review-20261025/`. The complete
release suite and installed IO/upgrade qualification are separate; no release,
package version change, staging, commit or push is part of this review.

GitHub inspection found no open Persistence issues or pull requests. Stable-map
and metric codec/continuity paths were inspected without a demonstrated runtime
defect; this bounded maintenance review is not a complete storage/security audit.
The independent PocketIC workspace and product-owned governance boundaries remain
intentional; no workspace split or new abstraction is introduced.

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
