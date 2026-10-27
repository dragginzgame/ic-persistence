# Current handoff

The released baseline is 0.51.6 at `b2b280c032fa01ec58d7b82c55320e243a0318e4`.
The pending compatible 0.51.7 batch implements the next six reviewed complexity
reductions. Public APIs, stored formats, allocation custody and release receipt
JSON remain unchanged. Package versions remain 0.51.6; this work is uncommitted
and the index remains user-owned. See [pending notes](../changelog/0.51.md#0517---unreleased).

## Current changes

- File-publication faults live with the filesystem substitute instead of the
  shared process/Git fault model. Failed fixtures retain both command state and
  file-fault state; existing partial-publication and recovery cases remain.
- Registry responses decode into typed required facts independently of network
  and publication orchestration. Extra registry fields remain accepted. Only
  HTTP 404 means absence; package identity, checksum and yanked status remain
  mandatory before a package counts as published.
- Archive qualification generates its workspace, patches and consumer manifest
  in one pass over the same extracted-package map, retaining graph admission and
  the native/MSRV/Wasm consumer roster.
- Fixture diagnostics bundle storage, recovery and page observations before
  caller policy. Report fields, callback ordering and corruption handling remain.
- Release adapter tests scope Make admission, jobserver, receipts and bootstrap
  in independent directories and subshells with one outer retention owner.
- Native process capture shares completion checks and error reporting. Metadata
  and text capture retain their distinct output budgets and stderr behavior.

Both selected lockfiles, dependency catalogs, compiler choices and tool pins are
unchanged from the clean incoming source. Shared Tooling remains the reviewed
0.26.20 snapshot at `a7f85d45d4212d4c39225f58597580560a333550`; vendored source is
untouched. This compatible cleanup requires no consumer migration or retained-data
conversion.

## Qualification and limits

All 65 release-helper tests and 62 selected runtime tests pass. They cover
publication conflicts and uncertain uploads, filesystem publication failures,
retained fixture evidence, diagnostic wire names, corruption and observations
before policy callbacks. Strict Clippy passes for Memory's library, tests and
release helper with test support enabled. ShellCheck and Bash syntax checks pass
for both changed scripts.

The adapter scenarios, fixture retention probes and archive graph-admission
scenarios pass. Their substituted compiler/Git effects do not qualify a live
release. A separate real archive-consumption run passes native package checks,
native consumer execution on Rust 1.99.1 and the selected 1.91 MSRV, and Wasm
consumer compilation. Source, archive and extracted-file hashes pass. Its
retained attempt is
`target/package-qualification/attempt.aKrWGn/`. This packages working-tree edits
with `--allow-dirty`; later documentation edits are outside those archives.

The combined generator also reproduces the previous three manifests byte for
byte using identical inputs, including paths containing spaces and quotes.
Logs and the isolated comparison script are retained at
`/tmp/ic-persistence-next-six/`; initial lint and sandbox subprocess failures
remain alongside successful retries. Formatting, local documentation references
and diff checks pass.

Full release validation, installed IO/upgrade qualification, performance
measurement and current macOS execution were not rerun for this cleanup. Wasm
consumer compilation is a build check, not execution of diagnostic callbacks.
No release, publication, commit, tag or push ran. The
[released handoff](https://github.com/dragginzgame/ic-persistence/blob/b2b280c032fa01ec58d7b82c55320e243a0318e4/docs/status/current.md)
retains the previous cleanup's measurements and earlier dependency, installed
runtime and supplier evidence with their original scope. This task performs no
fresh dependency-security audit. The [ownership guide](../ownership.md),
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
