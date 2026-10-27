# Current handoff

The released baseline is 0.51.12 at `41a04886ec29546e04fcec759821d5f818fd64ab`.
Memory and Storage 0.51.12 are published on crates.io, not yanked, and declare
Rust 1.91.0. Storage selects Memory `^0.51.12`. The singleton loader requested by
[#136 — Provide non-initializing StableCell loading for Blob singleton journals](https://github.com/dragginzgame/ic-persistence/issues/136)
is available to downstream consumers. Blob retains its own gateway, fence, sync,
callback and retained-state qualification; no Blob migration/reset ran here.

The pending compatible 0.51.13 batch repairs the remaining native CI fixture for
[#135 — Adopt shared formatter orchestration without dropping package-consumer formatting](https://github.com/dragginzgame/ic-persistence/issues/135).
Only the consumer test script, notes and handoff are edited. Package versions
remain 0.51.12; selected dependencies, both lockfiles, product APIs and stored
bytes are unchanged. The changes are uncommitted and the index remains untouched.
See [pending notes](../changelog/0.51.md#05113---unreleased).

## Live acceptance

[CI for the delivered source](https://github.com/dragginzgame/ic-persistence/actions/runs/40552091313)
passes the Linux native gate, tooling lint, all three MSRV jobs and installed
runtime qualification on Linux, Apple Silicon and Intel macOS, including
original and optimized Wasm.
The Make 3.81 parsing failure from 0.51.11 is resolved.

Both macOS native jobs now reach the formatter's compiler-selection fixture and
fail in `test_jobserver_handoff`. The environment-only case expands an empty
argument array under Bash 3.2 `set -u`, producing `compiler_arguments[@]: unbound
variable` in its redirected case log. Linux's newer Bash does not reject that
expansion. The native gate still needs to pass on the repaired delivered source
before acceptance closes; the owning issues retain this qualification boundary.

## Current repair and focused evidence

The fixture uses the repository's guarded empty-array expansion idiom, preserving
the distinction between environment-only and command-line compiler callers. All
five formatting passes still have to use the selected validation compiler.
Failure diagnostics now print compiler case logs alongside retaining the complete
fixture. The shared snapshot and formatter itself are untouched.

Evidence is retained at `/tmp/ic-persistence-issue-135-bash32/`:

- GNU Bash 3.2 built from GNU's original source reproduces the exact empty-array
  failure in the unmodified delivered harness (`before-harness.log` and retained
  `before-harness-fixtures/*/jobserver/compiler-environment.log`).
- The repaired consumer adapter fixture passes with that Bash 3.2 harness and
  GNU Make 3.81 (`after-harness.log`), including compiler selection, pass order,
  jobserver handoff, failure evidence and substituted release/Git effects.
  Child scripts use the installed host Bash: this is a focused shell/parser
  proof on Linux, not a fresh native macOS result.
- The same fixture passes with the installed Bash 5 and GNU Make 4.3
  (`after-bash5-make43.log`). Bash syntax, ShellCheck, documentation links and
  diff checks pass. Source/archive hashes and publication observations are
  retained with the logs. No broad Rust suite was rerun for this test-only fix.

Temporary GNU shell/Make sources and build attempts remain retained. Bash 3.2.57
source could not build without regenerating its stale distributed parser; the
original Bash 3.2 source supplies the bounded harness proof. No system shell,
selected tool pin or installed tool-store selection was changed.

Earlier cell-loader native/MSRV/Wasm proof remains at
`/tmp/ic-persistence-issue-136/`; earlier Make 3.81 proof remains at
`/tmp/ic-persistence-issue-135-make381/`. The
[released handoff](https://github.com/dragginzgame/ic-persistence/blob/41a04886ec29546e04fcec759821d5f818fd64ab/docs/status/current.md)
retains their original commands, source identities and scope. The current hosted
runtime results add all-host installed qualification without relabelling those
older local runs.

## Delivery boundary

Deliver the fixture repair, then verify the applicable native CI matrix before
closing acceptance. No CI rerun, commit, tag, push, release, publication or Blob
change ran in this continuation. Issue descriptions record the delivered source,
published API and remaining native gate. Shared Tooling remains 0.26.26 at
`5683060a5d8ce79af2fcfedb53b3eb24037fd064`; the reviewed
[adoption boundaries](../shared-tooling-adoption.md) remain active.

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
