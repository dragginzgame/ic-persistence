# Current handoff

The released source baseline is 0.51.2 at `c7114bc`. The compatible pending
0.51.3 cleanup changes local implementation and fixtures; package versions,
public APIs, stable formats, release receipt JSON and dependency selections stay
unchanged. Shared Tooling remains 0.26.16 at
`2077c934efb82b181bea58fe50078dbad289c1aa`.

## Current scope

Cold bootstrap returns its generation after durable commit and binding publication.
Allocation accounting uses named partitions. Publication keeps package/digest/registry
observations together and still observes every package before uploads. Package
identity/inheritance admission has one Rust implementation. Release fixtures share
semantic effect predicates, including the Bash archive adapter, while retaining
raw argument logs. Storage tests share fixture keys; the independent runtime runner
owns its local IO framing helpers.

Consumer Make tests retain alias dispatch, representative forbidden families,
hidden-MFLAGS refusal and local delivery/cache/order/failure policy. The selected
Shared Tooling source owns the exhaustive generic release-command matrix. No
shared payload or sibling checkout is patched. Moving repo-tool to a separate
package remains conditional on demonstrating simpler bootstrap and qualification.
See [the pending notes](../changelog/0.51.md#0513).

## Qualification

Strict Memory/Storage all-target and independent-runner Clippy pass. All 62
release-helper, 62 runtime, 12 Storage and three runner tests pass, plus the
accounting-fixture check; six opt-in allocation measurements were not run.
Both affected compiler-floor checks, Wasm probe budgets, policy/format/link checks,
the reduced consumer release/bootstrap fixtures and the exhaustive Shared Tooling
release-command tests from the exact adopted source pass.

Actual archive/native/MSRV/Wasm consumption and final integrity checks pass at
`target/package-qualification/attempt.HZwF3K/`. Installed Memory/Storage IO,
upgrade and rollback pass original/O3/Os/Oz at
`target/qualification/runtime/attempt.9IkhsT/`. Logs and the starting source/index
snapshot remain at `/tmp/ic-persistence-complexity-final/`. Public APIs, formats,
package versions, catalogs/lockfiles, compiler/tool pins, shared payloads and the
real index are preserved. Full release gates, hosted macOS qualification and
release/publication retain their separate authority.

## Retained evidence and recovery

The [committed 0.51.2 handoff](https://github.com/dragginzgame/ic-persistence/blob/c7114bcdcdb97282731116a8888f8432e6bd8484/docs/status/current.md)
preserves the earlier dependency/cleanup source identities, qualification paths,
supplier uncertainties and upstream references. Its pending-work descriptions
are historical. Earlier native archive and installed evidence remain
at `target/package-qualification/attempt.peGojm/` and
`target/qualification/runtime/attempt.iYaO7a/`; they do not qualify this new source.
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
