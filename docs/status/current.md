# Current handoff

The released baseline is 0.51.10 at `44736dccb2b9b0894df465bbba673c81da8007bb`.
The pending compatible 0.51.11 batch adopts Shared Tooling 0.26.26 and implements
[#135 — Adopt shared formatter orchestration without dropping package-consumer formatting](https://github.com/dragginzgame/ic-persistence/issues/135)
in the working tree. The upstream standalone capability is now supplied by the
selected committed source. Package versions remain 0.51.10; changes are
uncommitted and the index remains user-owned. See
[pending notes](../changelog/0.51.md#05111---unreleased).

## Current changes

Shared Tooling 0.26.26 at `5683060a5d8ce79af2fcfedb53b3eb24037fd064` is exported
from a clean detached committed source using the canonical impact preview and
refresh. The same 77 roots derive 123 payloads. The refreshed hook accepts opaque
gitlinks without reading child contents or changing their staged identities;
the required Node guide completes governance documentation without Node setup.
The Clap-based runtime scanner is not selected, so no compiler-package adaptation
or dependency change is needed. Gzip ownership guidance remains policy; Cargo/tar
packaging and evidence retain their infrastructure boundary.

The formatter now imports its declared `RustSource.pm` companion for balanced
attribute inspection in workspace-only cache admission. The hook fixture copies
and stages that module, and the jobserver fixture compares its canonical copy.
The cache fix avoids treating unrelated path text after conditional attributes as
module overrides. Persistence's standalone selection remains fully uncached, so
this does not enable result reuse. Generic lexer/cache fixtures remain owned by
Shared; source adoption does not claim a fresh producer or macOS qualification.

The Makefile now declares the existing extra workspace and the edition-2024
`testing/package-consumer.rs` source through `FORMAT_RUST_ARGS`. The paired
command overrides and copied adapter are retired. Shared directly runs grouped
manifest sorting and Rustfmt for the root, then for runtime qualification,
followed by standalone Rustfmt. Every pass reports live progress and elapsed time.
The validation compiler is exported for all formatter passes, including when a
caller supplies another rustup compiler on Make's command line.

Standalone selections run entirely uncached, preserving arbitrary module/config
coverage. Repeated successful checks still run every pass; explicit `--no-cache`
force-run routing remains supported. Write/check coverage, prepared-tool admission
and offline behavior remain unchanged. Both selected lockfiles, all dependency
catalogs, package versions, features, tool/compiler pins, public APIs and durable
formats are preserved.

The order change is intentional: working-tree `make fmt` was already
nontransactional, since earlier manifest edits could precede a later failure.
Check mode remains non-mutating. The hook formats an exported index and publishes
selected bytes only after the complete formatter succeeds. No cross-workspace
all-sorts-before-Rustfmt guarantee is required by these owned caller boundaries.

## Consumer qualification

Snapshot integrity/mode checks, Perl syntax for the driver and lexer, Bash syntax and
ShellCheck for the consumer fixtures pass. Root `make fmt-check` and real native
hook write/check flows pass with visible workspace and standalone output after
the lexer companion refresh. Both lockfiles
remain byte-for-byte equal to their incoming selections.

The complete consumer hook fixture passes with prepared native formatters on
Linux. It covers both workspaces, unformatted extra-workspace/standalone sources,
non-mutating rejection, standalone configuration changes after prior success,
staged writeback, partial-staging refusal, unrelated source/index preservation,
formatter failure isolation and setup. Fixture assertions retain the prohibition
on builds, lock creation or target/cache creation. No fixture commits/tags are
created; indexes reuse an existing commit read-only.

Consumer release-adapter scenarios pass using the actual shared driver with
substituted executable effects. They verify the selected compiler and exact new
pass order, jobserver descriptors through Cargo and standalone Rustfmt in write
and check modes, admission before workspace passes, late standalone failure,
explicit force-run behavior and complete retained logs with exact child exits 37
and 39 (Make reports its normal status 2). Hook/jobserver fixture copies include
the canonical formatter closure. These are consumer checks, not a replay of the
generic producer matrix.

Adoption source, incoming changes/locks/snapshot, impact/refresh output and logs
are retained at `/tmp/ic-persistence-shared-02626/`. Earlier full formatter
migration evidence remains at `/tmp/ic-persistence-shared-02624/`. Final scenario output records
the retained negative-case log paths. Earlier source-adoption and preparation logs
remain at `/tmp/ic-persistence-shared-02623/` and
`/tmp/ic-persistence-issue-135/`. Documentation links and diff checks pass.
Library source and dependency selections are unchanged, so Rust suites, installed
IO/upgrade, fresh performance measurement and the full release gate were not
rerun. Current macOS qualification was not repeated; hosted/native acceptance
retains its own scope.

No remote issue closure/comment, commit, tag, push, release, publication or hook
activation in the real checkout ran. #135's local implementation is ready for
review and delivery. The
[released handoff](https://github.com/dragginzgame/ic-persistence/blob/44736dccb2b9b0894df465bbba673c81da8007bb/docs/status/current.md)
retains earlier dependency, cleanup and installed-runtime evidence with its
original scope. The [adoption boundaries](../shared-tooling-adoption.md),
[ownership guide](../ownership.md) and [host support](../host-support.md) remain
active.

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
