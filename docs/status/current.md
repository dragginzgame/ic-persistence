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

Both packages inherit released workspace version 0.50.12 at
`a1cbd568e315f4f2f461f00a656738460ca79b8c`. Its complete
[hosted CI](https://github.com/dragginzgame/ic-persistence/actions/runs/39925986223)
passed. Historical consolidation, dependency and runtime qualification retains
its original scope in the numbered changelogs and
[runtime notes](../runtime-io-qualification.md).

Pending compatible 0.50.13 selects Host/Tooling 0.66.0 in the root catalog and
both helper CLIs. The root lock changes only five Tooling versions/checksums;
their Rust sources match 0.65.0. The independent lock is unchanged. Platform
requirements/selections, effective features, compiler identities, substrate 0.7.2,
PocketIC client/server 16.1.0, public APIs and stored formats remain selected.
The PocketIC owner-transfer API change is outside this consumer's calls.

Shared 0.26.8 is adopted at `0b07c2a3163a5a66328e100b553baf93afd8ed07`
through canonical preview/refresh (75 roots, 114 payloads). Cache bootstrap no
longer takes a global installation lock; selected-tool slots and bounded waits
still serialize actual builds. The policy digest changes, so explicit setup has
prepared the common tools and both 0.66.0 CLIs under the new identity. Offline
admission passes. Older generations, preparation slots and failure evidence are
retained. The complete producer Rust installer and bounded-wait fixtures pass.

Tooling source `d71f4690c7e8e3ffe99f72467bc35cbadce33042` has passing optimizer,
installed runtime and browser workflows. Its
[native matrix](https://github.com/dragginzgame/ic-tooling/actions/runs/39930549305)
passes Linux and MSRV, with both macOS jobs pending at review. Shared's
[current CI](https://github.com/dragginzgame/shared-tooling/actions/runs/39928847334)
passes Linux and ARM macOS, with Intel macOS pending. The earlier #302 Intel installer fixture passed; that job failed
later on an artifact-download HTTP 503. It remains failed evidence, not native
acceptance. #301 stays open until applicable hosted qualification passes.

Focused Linux qualification passes all-target Clippy, actual Rust 1.91 compilation,
pin/production-graph checks, three selected release metadata/setup tests, nine
native IO/accounting tests, release/bootstrap, hooks and failure-evidence/
retention fixtures, formatting, tooling lint and documentation links. All four
installed Wasm variants pass at `target/qualification/runtime/attempt.eheuzs/`;
the 36 IO/instruction rows match the retained 0.65.0 log. Logs remain at
`/tmp/ic-persistence-shared0268-host066/`. Full release and hosted consumer/native
acceptance retain their separate scope.

The delivered tooling requires inherited member manifests and matching local lock
versions before bootstrap. Archive qualification consumes the checked catalog
MSRV. Runtime CI cache keys bind tool selections and Shared's build identity.
CI retains its failure-evidence fixture for the hosted archive round trip;
normal packaging and archive-consumer qualification remain independent gates.

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
