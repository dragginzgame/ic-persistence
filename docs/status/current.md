# Current handoff

The released baseline is 0.51.8 at `5bce9fa85577aa5041c79b601fe1642f936992ea`.
The pending compatible 0.51.9 batch adopts Platform 0.7.0, Tooling 0.68.12 and
Shared Tooling 0.26.22. Public APIs, stored formats, allocation custody and release
receipt JSON remain unchanged. Package versions remain 0.51.8; work is
uncommitted and the index remains user-owned. See
[pending notes](../changelog/0.51.md#0519---unreleased).

## Current changes

Both workspace catalogs select published Platform 0.7.0 at
`60c43108ca7f2a99bfac62ec21075afff5ed7a7e`. Its removed Candid compiler exports
are unused here. The selected stable-structures and PocketIC facades are unchanged
from the previous release. Underlying public type identities remain registry
stable-structures 0.7.2 and PocketIC 16.1.0, so no Persistence consumer migration,
client regeneration or retained-data conversion is required.

The five native Tooling requirements and release-editor/Testkit CLI pins select
published 0.68.12 at `42a9ec79b10f7bf9e3a5658980ca73817c2a771d`. The incoming root
lock already selected Platform 0.6.21 and part of Tooling 0.68.12. Those identities
are preserved; Build and Host Process complete the 0.68.12 selection, and direct
library dependencies add Platform 0.7.0. Native Testkit still requires Platform
0.6.21 transitively. Both facades share one exact stable-structures identity;
the default production graph contains one Platform identity, at 0.7.0. The
independent lock changes only Platform from 0.6.20 to 0.7.0. Other dependency
selections, compiler pins, features/defaults and Rust 1.91 remain unchanged.
Metrics remains the latest published 0.16.3. Registry observations confirm the
selected releases are not yanked, and packaged VCS records match the commits above.

Shared Tooling 0.26.22 at `5fc904a935d82ad31f0fba14434b71ccb8c10345` is exported
from a clean detached copy through the canonical impact preview and refresh.
All 77 roots and 120 payloads remain selected. Six governance payloads change;
no shared executable, common tool pin or installer build-policy identity changes.
The adopted rules make owned CandidType restrictions explicit and strengthen
native error, strict machine-output, bounded-process, safe terminal and required
source-inspection review. Current root notes already use the required emoji
categories. A strict-read scan of both maintained Rust crate trees finds no
CandidType tokens. This is source/policy adoption, not new automated enforcement
or a repository-wide code-hygiene audit.

## Qualification and limits

Snapshot integrity, dependency pin/inheritance, default production Wasm graph,
external IC dependency boundaries and formatting pass. Strict Clippy passes for
all three libraries, their tests and the release helper with test support.
All 65 release-helper, 62 selected runtime and 13 Storage unit tests pass, as do
Storage doctests. This includes native Testkit memory wrappers used with the
new library facade, retained-region behavior, rollback, uncertain publication
and receipt recovery. The independent runtime host package checks offline with
Platform 0.7.0 and PocketIC 16.1.0.

Consumer release-adapter scenarios pass with substituted Git/build effects and
the newly selected real release editor. Explicit tool setup prepares/adopts both
0.68.12 CLIs, retaining prior generations. One Testkit setup wait timed out while
another repository held the selected store slot; the retained retry reuses the
completed generation. No foreign owner or lock was changed. Subsequent offline
release-editor and Testkit/server admission pass. The server remains 16.1.0.

Actual extracted-package checks, consumer execution on Rust 1.99.1 and the
selected 1.91 MSRV, and Wasm consumer compilation pass. Source, archive and
extracted-file hashes pass. Its retained attempt is
`target/package-qualification/attempt.HkQAiQ/`. This qualifies working-tree
archives with `--allow-dirty`; final documentation edits follow that packaging
run. Local documentation references and diff checks pass.

Registry observations, incoming locks/snapshot, clean Shared source, impact and
refresh output, the initial busy-tool failure and successful retries, and focused
qualification logs are retained at `/tmp/ic-persistence-adopt-0519/`.
Full release validation, installed IO/upgrade qualification, fresh performance
measurement and current macOS execution were not rerun. The unchanged client/
server pair does not require a new pairing qualification; server admission is
not an installed IO/upgrade run. No release, publication, commit, tag or push ran.
The
[released handoff](https://github.com/dragginzgame/ic-persistence/blob/5bce9fa85577aa5041c79b601fe1642f936992ea/docs/status/current.md)
retains previous cleanup, dependency and installed-runtime evidence with its
original scope. This task performs no fresh dependency-security audit. The
[ownership guide](../ownership.md), [host support](../host-support.md) and
[adoption boundaries](../shared-tooling-adoption.md) remain the active contracts.

## Retained evidence and recovery

All original Memory/Storage issues, comments and PR histories remain preserved
through the [migration maps](../migration/ic-persistence.md) and
[historical exports](../history/pull-requests/README.md). Preserve the verified
backups at `/home/adam/projects/ic-persistence-retirement-2026-10-20/` and all
mirrors, exports and recovery plans. The original checkouts were retired with
maintainer authorization. Last recorded GitHub deletions were refused because
the CLI token lacked `delete_repo`; no deletion retry is part of this work.
Earlier package/runtime archives, receipts, failed attempts and logs retain their
original custody and source identities.
