# Current handoff

The released baseline is 0.51.7 at `1fd41a43bb661d007ab45c02b871963b89085f7f`.
The pending compatible 0.51.8 batch adopts published Tooling 0.68.11 and Platform
0.6.20 and completes the next six reviewed complexity reductions. Public APIs, stored formats, allocation custody and release receipt JSON
remain unchanged. Package versions remain 0.51.7; work is uncommitted and the
index remains user-owned. See [pending notes](../changelog/0.51.md#0518---unreleased).

## Current changes

Both workspace catalogs select Platform 0.6.20. The five native Tooling
requirements and release-editor/Testkit CLI pins select 0.68.11. Metrics remains
the latest published 0.16.3. Registry observations confirm these releases are not
yanked and declare Rust 1.91. Published source identities are recorded in the
pending notes; unpublished Platform 0.6.21 and dirty supplier source are excluded.
Shared Tooling remains the reviewed 0.26.20 snapshot at
`a7f85d45d4212d4c39225f58597580560a333550`; vendored source is untouched.

The incoming root lock already selected the new releases and is preserved byte
for byte. The independent runtime lock changes only Platform from 0.6.19 to
0.6.20. Feature/default selections, compiler pins and the Rust floor are unchanged.
The substrate remains registry stable-structures 0.7.2 and PocketIC client/server
remain 16.1.0. No consumer migration, client regeneration or retained-data
conversion is required.

Explicit locked fetches prepare the two caches separately from offline checks.
Explicit tool setup prepares the selected CLI generations; older generations,
servers and failed attempts retain their existing custody. Ordinary admission
and validation do not install tools or fetch dependencies.

Release edits keep backups beside expected file bytes and retain manifest-last
publication and owned-byte rollback. Receipt supersession separately admits old
intent/ancestry and preserves earlier receipts before replacing active evidence.
Test effects give Git, build and registry their own state and faults, retaining
one ordered command log and file-fault evidence. Archive admission shares pure
graph predicates with standalone fixtures; the real script still checks graphs
before compiler dispatch. Diagnostic exports share page-size projection while
fixture observations remain before policy callbacks. Metric continuation carries
admitted differences once across aggregate, summary and histogram checks, keeping
its allocation-free loop, exact-count and sticky-overflow rules.

## Qualification and limits

Dependency pin/inheritance, default production Wasm graph, external IC dependency
boundary, release adapter dispatch and offline tool/server admission passed for
the preceding dependency adoption. Their selected inputs are unchanged by the
cleanup. Both lockfiles remain byte-for-byte identical to this cleanup's incoming
state, including the earlier Platform-only independent lock update.

All 65 release-helper, 62 selected runtime and 28 Metrics tests pass after the
cleanup, along with focused diagnostic export/serialization checks. Strict Clippy
passes for affected libraries, tests and the release helper with test support;
the final metric helper also passes its package's strict gate. Coverage retains
partial writes, rollback, receipt recovery, tag/publication ordering, callback
observation timing, V1 byte mutations and exact/saturated checkpoint boundaries.
ShellCheck, Bash syntax and pure archive graph tests pass. The archive integration
probe rejects before compilation and retains selected-MSRV dispatch coverage.

Actual extracted-package checks, consumer execution on Rust 1.99.1 and the
selected 1.91 MSRV, and Wasm consumer compilation pass. Source, archive and
extracted-file hashes pass. The retained attempt is
`target/package-qualification/attempt.SBXTcO/`. This qualifies working-tree
archives with `--allow-dirty`; later changelog/handoff and pure-fixture edits
follow that packaging run. Local documentation references, formatting and diff
checks pass. No release, publication, commit, tag or push ran.

Cleanup logs, incoming lock preimages and isolated measurement sources are at
`/tmp/ic-persistence-six-0518/`. Both measurement workspaces use the current
selected catalog/lock, identical probe instrumentation and copied package source;
the baseline metric implementation is the incoming 0.51.7 source. Native
interleaved comparisons cover 0, 1, 4, 64 and 256 histogram bounds with unchanged,
appended and new-maximum states. An initial candidate slowed small histograms;
the retained helper's inline hint removes that cost in the measured cases. Final
median ratios range from 0.80 to 1.02, with no material measured slowdown. The
isolated pairwise comparison agrees on 14 aggregate, 23 summary and 10 histogram
states, complementing maintained boundary and byte-mutation tests.

Identical Wasm probes exercise aggregate, summary and 256-bound histogram
continuation in Node/V8. Final median ratios are about 1.02, 0.88 and 0.99
respectively; the combined probe decreases from 22,226 to 21,778 bytes. These
are engine timings and isolated probe sizes, not IC instruction measurements or
whole-application guarantees. Initial candidates, harness/lint failures and
successful retries remain retained. No production benchmark mode is added.

Earlier dependency observations, setup and qualification logs remain at
`/tmp/ic-persistence-deps-0518/`. Full release validation, installed IO/upgrade
qualification and current macOS execution were not rerun. The unchanged
PocketIC 16.1.0 server's admission is not an IO/upgrade run. The
[released handoff](https://github.com/dragginzgame/ic-persistence/blob/1fd41a43bb661d007ab45c02b871963b89085f7f/docs/status/current.md)
retains earlier cleanup measurements, installed-runtime and supplier evidence
with their original scope. This task performs no fresh dependency-security audit.
The [ownership guide](../ownership.md), [host support](../host-support.md) and
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
