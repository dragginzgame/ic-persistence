# Current handoff

The released baseline is 0.51.4 at `cf2daeb`. PR
[#134 — Use IcyDB-style sections in the latest root changelog](https://github.com/dragginzgame/ic-persistence/pull/134)
was squash-merged as `cb610c5b75861b76408cda4cf1cefb2e68bd8484`; this checkout
is fast-forwarded to it. The pending compatible 0.51.5 batch adopts the latest
published upstream dependencies. Package release versions, public library APIs,
stable formats and release receipt JSON remain unchanged. Adoption edits remain
uncommitted; staging remains user-owned.

## Current selection

Both catalogs select Platform 0.6.19 at
`0502bce432c2478cf86215435864223a22a14a38`. The five native Tooling libraries
and release-editor/server CLI selections use 0.68.8 at
`7c3b715d07bbeca38b4ba89a60d3bec309de7417`. Metrics is 0.16.3 at
`27a46f1076ee757680b6e6b1974947e482f3158d`, within the existing 0.16 requirement.
The incoming root lock already selected all of these and is preserved exactly.
The independent lock changes only Platform's version/checksum.

Supplier comparisons show no Rust package-source changes from Tooling 0.68.7,
Platform 0.6.18 or Metrics 0.16.2. Platform's increased optional Wire metadata
minimum is not enabled by either consumer graph. Root Platform features remain
`stable-structures`; the independent runner uses `pocket-ic`/`principal`.
The substrate remains registry stable-structures 0.7.2 and PocketIC client/server
remain 16.1.0. Compiler and default-feature selections are unchanged.

Shared Tooling is already at the latest committed 0.26.20 revision,
`a7f85d45d4212d4c39225f58597580560a333550`: 77 roots and 120 payloads.
Uncommitted sibling governance edits are excluded. Root notes retain PR #134's
emoji Summary layout. See [pending notes](../changelog/0.51.md#0515---unreleased).
Normal releases start fresh; recovery uses `release-resume VERSION=X.Y.Z`.
Consumer receipts, archives and failed logs retain their original custody.

## Qualification and limits

Snapshot/pin, production graph, external dependency boundary and formatting
checks pass. The focused release-helper version/note preservation test passes
against the new native libraries. Both 0.68.8 CLI installations reused verified
shared-store generations and the retained PocketIC server.
Metrics Stable's unit tests and doctests pass with Metrics 0.16.3. The selected
release editor admits both interrupted metadata states in
`/tmp/ic-persistence-bootstrap.kevBmR/`. Installed Memory/Storage IO, upgrade and
rollback pass original/O3/Os/Oz at `target/qualification/runtime/attempt.rL4O4P/`.
Current setup and qualification logs are at `/tmp/ic-persistence-adopt-0515/`.

PR #134's documentation diff and author access were reviewed before merge.
No branch protections or rules required the remaining hosted jobs to complete.
Its [CI run](https://github.com/dragginzgame/ic-persistence/actions/runs/40361811321)
still had pending native/runtime jobs at merge; those are not passing evidence.
Full release validation and current macOS qualification remain separate.
No release, publication or dependency-adoption commit is performed.

## Retained evidence and recovery

The [0.51.4 handoff](https://github.com/dragginzgame/ic-persistence/blob/cf2daeb/docs/status/current.md)
preserves prior 62-test helper, installed IO/upgrade/rollback, native/MSRV/Wasm
archive qualification and supplier advisory observations with their exact inputs.
Logs remain at `/tmp/ic-persistence-adopt-20261026/`,
`/tmp/ic-persistence-upstream-next/`, `/tmp/ic-persistence-deps0685/` and
`/tmp/ic-persistence-platform-shared/`. Earlier package/runtime attempts remain
retained; they do not independently qualify the new dependency identities.
This task does not perform a fresh dependency-security audit.
The [ownership guide](../ownership.md), [host support](../host-support.md) and
[adoption boundaries](../shared-tooling-adoption.md) remain the active contracts.

All 121 original Memory/Storage issues, 362 comments and 23 PR histories were
preserved through the [migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). The maintainer authorized
retiring both original checkouts and repositories. The old local checkouts were
removed after verified backups at
`/home/adam/projects/ic-persistence-retirement-2026-10-20/`. Last recorded GitHub
deletions were refused because the CLI token lacked `delete_repo`; no deletion
retry is part of this work. Preserve the backups, mirrors, exports and recovery
plans. The [original consolidation evidence](https://github.com/dragginzgame/ic-persistence/blob/4dda4c1511f09f6bd14d1ea8ae8d428dccdd2f75/docs/status/current.md#historical-consolidation-qualification)
retains its original source scope.
