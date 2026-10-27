# Current handoff

The released baseline is 0.51.5 at `e51d6588f889ac491f7177221c0bcf2a17f6f6ec`.
The pending compatible 0.51.6 batch implements the six reviewed complexity
reductions. Public APIs, stored records, allocation custody, release receipt JSON
and package release versions remain unchanged. Work is uncommitted and the index
remains user-owned. See [pending notes](../changelog/0.51.md#0516---unreleased).

## Current changes

Release preparation and commit checks reuse source-derived expected metadata
within the current operation. Live source, receipt, configuration, compiler,
lockfile, working-tree and archive checks remain around their existing effects.
One receipt owner collects named package digests; callers carry complete evidence
instead of unpacking parallel arrays. Receipt fields and archive order remain
unchanged, including unknown-field refusal and retained artifact custody.

Receipt-writer tests use passive receipt inputs without a simulated release.
Full release/recovery tests retain ordering and interruption coverage. Shared
runtime-test declarations and pools now have one fixture module; test suites no
longer import those helpers from each other. Production bootstrap is unchanged.

Histogram continuation separates bucket metadata and lazy interval projection
from admission/accounting while retaining its indexed allocation-free loop.
Storage and Metrics Stable derive mechanical error plumbing, preserving error
variants, messages, conversion behavior and typed sources. They add only direct
edges to the already-selected workspace Thiserror dependency.

The incoming root lock's Tooling 0.68.9 and rustix 1.1.6 selections are preserved.
Its only task-added changes are the two Thiserror edges. Neither dependency
minimums, CLI pins, the independent workspace/lock nor compiler choices change.
Shared Tooling remains the reviewed 0.26.20 snapshot at
`a7f85d45d4212d4c39225f58597580560a333550`; vendored source is untouched.

## Qualification and limits

Focused strict Clippy passes for the affected libraries, tests and release
helper; the final histogram source also passes its package's strict lint gate.
All 62 release-helper, 62 selected runtime, 28 Metrics and 13 Storage unit tests
pass, along with three Metrics doctests. This includes source/receipt conflicts,
rollback and uncertain publication, sticky overflow and exact-count continuation,
lease behavior and typed error sources. Dependency pin, production graph,
external IC boundary and formatting checks pass.

Logs, incoming lock preimage and isolated measurement sources are retained at
`/tmp/ic-persistence-simplify-six/`. The baseline is the released source above;
measurement crates add identical temporary test/export instrumentation outside
maintained source. Interleaved native comparisons cover 0, 1, 4, 64 and 256
histogram bounds with unchanged, appended and new-maximum states. No material
slowdown was observed in these cases; they are not IC instruction measurements.
The histogram-only Wasm probe stays at 25,355 bytes. The combined histogram/error
probe grows from 25,355 to 27,085 bytes with derived error plumbing; its format and
source projections match the baseline on eight boundary inputs. The existing
Storage qualification Wasm remains 291,774 bytes. Earlier rejected histogram
variants and initial lint failures remain retained separately.

Full release validation, installed IO qualification and current macOS execution
were not rerun for this cleanup. No release, publication, commit, tag or push ran.
The [released handoff](https://github.com/dragginzgame/ic-persistence/blob/e51d6588f889ac491f7177221c0bcf2a17f6f6ec/docs/status/current.md)
retains earlier dependency, installed-runtime and supplier CI/advisory evidence
with its original scope. This task performs no fresh dependency-security audit.
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
