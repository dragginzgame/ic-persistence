# Current handoff

The local release baseline is 0.50.17 at
`e0a32af`, including the Metrics Stable import alongside Memory and Storage.
[The import guide](../migration/ic-metrics-stable.md) records its source and
cutover obligations; arithmetic `ic-metrics` stays external. Previous source,
merge and installed qualification scopes remain in
[the release-time handoff](https://github.com/dragginzgame/ic-persistence/blob/e0a32af/docs/status/current.md)
and the published 0.50 notes.

The pending 0.51.0 batch adopts IC Tooling 0.68.1 at
`c9c5ef7c44e3d0c3b23c8531a5d43c1800656c0a` for all five native development
dependencies and both helper CLIs. The incoming root lock already selected those
five versions/checksums; the subsequent Metrics adoption changes only its own
package version/checksum. The independent runtime
lock is unchanged. All six consumed packages, including the two CLI owners,
have identical published Rust sources to 0.68.0. Platform remains 0.6.15,
stable-structures 0.7.2 and PocketIC client/server 16.1.0. Optional Wire support
remains off; APIs, allocation custody, V1 metrics records and collection formats
are unchanged. The Shared snapshot remains
`4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.

Both selected CLIs reuse admitted installations without compilation. Explicit
setup, release-editor admission and PocketIC server admission pass with the
existing selected server. Full locked metadata for both maintained graphs,
pin/inheritance checks, production singleton/exclusion checks, declaration
boundaries, affected Memory all-target Clippy, all 62 release-helper tests,
Rust 1.91 all-target compilation and formatting pass. Evidence is retained at
`/tmp/ic-persistence-deps0681/`; this does not claim a new full release
gate, installed IO/upgrade run or macOS qualification.

At inspection, the supplier's Linux native, MSRV, lint/security, optimizer,
PocketIC and browser/session jobs pass. Both macOS native jobs remain in progress
in [the supplier CI run](https://github.com/dragginzgame/ic-tooling/actions/runs/40103517968).
No shared snapshot adoption, workspace version bump, release or Git effect is
performed by this dependency change.

## Metrics 0.16 adoption

The pending minor release also selects published IC Metrics 0.16.0 at
`c0f033a8f41c280fd0fc77b2ec934d2f2b8176ed`. All 15 published Rust source files
match 0.15.8; there are no production dependencies. The root lock changes only
Metrics' version/checksum from the preceding Tooling adoption state. The
independent runtime graph has no Metrics dependency and remains unchanged.

This raises the pending candidate from 0.50.18 to 0.51.0 because the adapter's
public arithmetic/state-error types now belong to the distinct 0.16 Cargo
identity. Callers must adopt `ic-metrics = "0.16"` together with the adapter.
V1 codec bytes, continuity checks and storage formats remain unchanged.
The earlier Tooling notes move into the same pending minor; released notes and
package versions are untouched. Full locked metadata for both workspaces,
pin/graph/declaration policy, native and Wasm all-target adapter Clippy, all 27
adapter regression tests, three doctests, Rust 1.91 all-target compilation,
formatting and local documentation links pass. Focused evidence is retained at
`/tmp/ic-persistence-metrics016/`.

The supplier's Linux and macOS ARM native checks and MSRV pass at inspection;
macOS Intel remains in progress in
[the supplier run](https://github.com/dragginzgame/ic-metrics/actions/runs/40104719356).

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
