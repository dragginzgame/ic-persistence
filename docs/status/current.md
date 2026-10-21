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

Both packages inherit released workspace version 0.50.9. The completed 0.50.0
conversion needs no version exception or manifest-rewrite path. Shared Tooling
0.26.2 (`1959ef75d0b86ed7725d41ae5056303d35eb633d`) owns optimizer-only setup
and evidence. Release 0.50.8 passed all ten native/MSRV/installed-runtime checks;
the supplying Tooling 0.63.2 matrix also passed after its Intel DNS retry.

Release 0.50.9 delivers both crates and the independent runner
selecting published Platform 0.6.5. All maintained library/tests/example imports
use its exports. Direct substrate declarations and allowances are retired;
production admission checks exact registry 0.7.2 identity and singleton graphs.
Memory retains layout/growth safeguards; changes stop at the Platform boundary.

Local qualification passes strict Clippy for both workspaces, 207 product tests,
13 doctests, all 60 release-tool tests, adapter/bootstrap fixtures and MSRV/Wasm
checks. Actual Rust 1.91 library execution passes. Drift/failing-observation
fixtures exercise the protocol guard. All 40 native IO, 86 allocation and 36
installed valid-IO rows match the prior qualified baseline.

Installed original/O3/Os/Oz IO/upgrade evidence is retained at
`target/qualification/runtime/attempt.hElm2V/`; see
[runtime notes](../runtime-io-qualification.md). Both locks change only intended
Platform selections and consumer edges; PocketIC remains 16.1.0.

[Platform #45](https://github.com/dragginzgame/ic-platform/issues/45) supplies the
manager exports. Its exact release CI is still asynchronous; installed-runtime
producer acceptance passes. The adoption was delivered at
`7ac7f78c7b8846c65c40a73a3fb10a3f824bdd63`; remaining native acceptance is tracked
under
[Persistence #130](https://github.com/dragginzgame/ic-persistence/issues/130).

Pending compatible 0.50.10 tooling cleanup requires both inherited package
manifests during bootstrap while preserving interrupted metadata recovery.
Testkit's unchanged CLI pin lives in `ci/runtime-tools.env`; runtime CI keys
use that selection and IC pins with Shared's build identity, so unrelated
Makefile changes no longer select an empty tool cache.

CI retains its failure-evidence fixture from the main validation pass for the
hosted round trip. Archive rejection cases share prepared, hashed archives;
normal packaging and archive-consumer qualification remain independent gates.

Storage's original handoff and changelog remain under
[historical evidence](../history/ic-storage/handoff.md). Remaining technical
blockers and upstream acceptance belong to the linked GitHub issues rather than
another local queue.

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
