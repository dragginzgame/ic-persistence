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

Both packages inherit released workspace version 0.50.10. Its complete ten-job
native/MSRV/installed-runtime matrix passed at
`fbe1208518950c8f81a8dc8475852d7c330fcaa7`. The Platform boundary adoption in
[Persistence #130](https://github.com/dragginzgame/ic-persistence/issues/130)
is delivered and closed. Pending compatible 0.50.11 selects Shared Tooling
0.26.6 at `4cb9bc907154225a3f7e4c33fd8d75fbeaa125c8` through canonical export,
with 75 roots and 114 payloads. The CI/release integrity audit is added as
inspection guidance. Shared's selected-source CI passes; catalog pins and
consumer workflow coverage are unchanged.

Both workspaces retain Platform 0.6.7. The root catalog and selected helper CLIs
advance to Tooling 0.64.0 at `036da0cdf3d3b35297fbc26031e373f386a2f2de`.
The consumed Build/Host/native Testkit library Rust sources match 0.63.3; removed
Testkit forwarding/policy APIs are not called here. The root lock changes only
those five versions/checksums from the incoming graph; the independent lock is
unchanged by this adoption. Compiler identities, features, substrate 0.7.2 and
PocketIC client/server 16.1.0 remain selected.

Ordinary hosted integration follows the new asynchronous baseline; release and
issue-specific acceptance remain separate. Tooling 0.64's Testkit runtime CI
passes; its native matrix is pending. No publication or hosted acceptance of
these working-tree changes is claimed. Previous focused 0.63.3 qualification is retained at
`target/qualification/runtime/attempt.nNsun8/` and
`/tmp/ic-persistence-deps0633/` with its exact source/version scope.

Current local qualification passes strict Clippy, Rust 1.91 all-target compilation,
60 release-tool and nine Memory IO/accounting tests, release/bootstrap/admission,
hooks, formatting, snapshot/pin/graph checks and failure-evidence retention.
Both 0.64.0 CLIs are prepared/admitted. Installed original/O3/Os/Oz IO/upgrade
checks pass at `target/qualification/runtime/attempt.xqpBJd/`; all 36 IO rows and
four Wasm artifacts match the prior 0.63.3 baseline. Logs are retained at
`/tmp/ic-persistence-shared0266-host064/`. Full release and hosted native
qualification remain separate.

The pending tooling cleanup rejects split crate versions before bootstrap and
copies inherited member manifests without rewriting them. Archive qualification
uses the MSRV returned by compiler admission for both fixture metadata and the
consumer compiler, preserving the workspace catalog's authority. Admission and
recovery fixtures pass. Real development archives pass native execution on both
compilers and the Wasm check with unchanged source/archive hashes; evidence is
at `target/package-qualification/attempt.wxlFXl/` and
`/tmp/ic-persistence-cleanup0511/`.

The 0.50.9 qualification retains its exact source/version scope: strict Clippy,
207 product tests, 13 doctests, 60 release-tool tests and MSRV/Wasm checks passed.
All 40 native IO, 86 allocation and 36 installed valid-IO rows matched their
previous baseline. Installed original/O3/Os/Oz IO/upgrade evidence remains at
`target/qualification/runtime/attempt.hElm2V/`; see
[runtime notes](../runtime-io-qualification.md). These historical observations
are not relabelled as executions of the pending dependency update.

The delivered 0.50.10 bootstrap requires both inherited package manifests and
retains interrupted metadata recovery. `ci/runtime-tools.env` owns the selected
Testkit CLI; runtime cache keys bind tool selections and Shared's build identity
rather than unrelated Make recipes.

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
