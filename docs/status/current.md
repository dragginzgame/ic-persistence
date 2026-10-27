# Current handoff

The released baseline is 0.51.9 at `1f7d96f60ac00f4b13a8e1de0496a71b16bbeafa`.
The pending compatible 0.51.10 batch adopts published IC Tooling 0.69.0 in the
five native development dependencies and both helper CLIs. Public library APIs,
stored formats, allocation custody and release receipt JSON remain unchanged.
Package versions remain 0.51.9; work is uncommitted and the index remains
user-owned. See [pending notes](../changelog/0.51.md#05110---unreleased).

## Current changes

Tooling 0.69.0 is published from
`f268aef5d205e51811e934648afcd13f026635f1`. Registry observations confirm the
selected packages are not yanked and declare Rust 1.91; the packaged VCS record
matches this commit. Tooling's breaking change concerns Platform-owned Principal
values in Host Tools registry APIs, which Persistence does not consume. Our
existing Platform 0.7 requirement already meets its migration requirement.

The root lock updates only the five Tooling packages, removes the obsolete native
Platform 0.6.21 facade and normalizes references to the remaining Platform 0.7.0
identity. Other package selections and the independent runtime lock are unchanged.
Feature/default selections, compiler pins and the Rust floor remain unchanged.
The stable-memory substrate remains registry stable-structures 0.7.2, and PocketIC
client/server remain 16.1.0. No consumer migration, client regeneration or
retained-data conversion is required for Persistence.

Shared Tooling remains the reviewed 0.26.22 snapshot at
`5fc904a935d82ad31f0fba14434b71ccb8c10345`; vendored source and common tool pins
are untouched. Explicit setup reuses prepared 0.69.0 release-editor and Testkit
CLI generations and the existing 16.1.0 server, retaining older generations and
failed attempts. Ordinary checks remain offline and do not install tools.

## Qualification and limits

Dependency pin/inheritance, default production Wasm graph, external IC dependency
boundaries, formatting and strict Clippy for Memory's library, tests and release
helper pass. All 65 release-helper and 62 selected runtime tests pass, covering
release/recovery evidence and the newly unified native Testkit memory graph.
Consumer release-adapter scenarios pass with substituted Git/build effects and
the new real release editor. Offline release-editor and Testkit/server admission
pass.

Actual extracted-package checks, consumer execution on Rust 1.99.1 and the
selected 1.91 MSRV, and Wasm consumer compilation pass. Source, archive and
extracted-file hashes pass. The retained attempt is
`target/package-qualification/attempt.yE3e9I/`. This qualifies working-tree
archives with `--allow-dirty`; the final handoff edit follows that packaging run.
Local documentation references and diff checks pass.

Full release validation, installed IO/upgrade qualification, fresh performance
measurement and current macOS execution were not rerun. The PocketIC pair is
unchanged; server admission is not an installed IO/upgrade run. No release,
publication, commit, tag or push ran.

Registry observations, incoming locks, setup and qualification logs are retained
at `/tmp/ic-persistence-tooling-next/`. The
[released handoff](https://github.com/dragginzgame/ic-persistence/blob/1f7d96f60ac00f4b13a8e1de0496a71b16bbeafa/docs/status/current.md)
retains earlier source adoption, cleanup measurements, dependency and installed
runtime evidence with its original scope. This task performs no fresh dependency
security or repository-wide hygiene audit. The [ownership guide](../ownership.md),
[host support](../host-support.md) and
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
