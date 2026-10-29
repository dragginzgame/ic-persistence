# Current handoff — Logging Stable publication preparation

The maintainer selected a separate publication PR on 2026-10-29, following
installed qualification [PR #138](https://github.com/dragginzgame/ic-persistence/pull/138).
This isolated branch starts at its head
`1928117a04655cfc3319060379eae682b2f12e8c`, incorporating released Persistence
0.54.1. The contribution is stacked on #138; merge that qualification first and
retarget this branch to main. Workspace versions remain 0.54.1 and compatible
notes remain 0.54.2. No actual merge, release or registry upload is selected.
The normal Persistence checkout is untouched.

## Publication contract

The canonical `packages.json` roster now selects Memory, Storage, Metrics Stable
and Logging Stable, in dependency order. All four participate in coordinated
version projection, interrupted metadata bootstrap, archive qualification,
prepared/final receipts, checksum reconciliation and publication. The retired
unpublished roster and its alternate reader mode are removed.

Every package receipt requires `logging_sha256` alongside the three existing
archive digests. Missing/unknown receipt fields refuse; no default, converter or
legacy reader is introduced. Retain older saved-release receipts and resume
those releases using their original source and tooling. Registry observation
admits all four names/checksums before any upload. Logging uploads last; a refused
final upload retries only that package, and a lost successful response reconciles
its checksum without uploading it again. Existing identity/tag checkpoints and
immutable retained archives remain authoritative.

The extracted archive consumer now selects all four exact unpacked packages and
the public Logging type family from the Logging Stable manifest. It exercises
shared governed lease custody, Logging checkpoint reopen and sequence/time
watermarks after pruning all rows. Admission still refuses substituted package
paths and dependency drift before compilation. Rust 1.91, the selected compiler
and Wasm remain the archive consumer targets.

## Qualification and retained evidence

Publication preparation changes no production runtime code, persisted V1 record,
package version, dependency, shared snapshot or independent runtime graph.
The parent PR's installed Logging source and evidence remain applicable. Its
[CI run 40873315791](https://github.com/dragginzgame/ic-persistence/actions/runs/40873315791)
passes all original/O3/Os/Oz runtime jobs and MSRV on Linux and both macOS hosts.
The original local probe remains at
`/tmp/ic-logging-stable-20261029/target/qualification/runtime/attempt.pUTvdW/`.
The parent's handoff and earlier evidence are retained in Git history and its
isolated checkout; do not relabel those receipts as publication evidence.

Current publication logs use `/tmp/ic-logging-publication-*.log`; package archive
qualification retains its own manifests, lock admissions, archives, compiler
outputs and checksums under this checkout's `target/package-qualification/`.
The positive four-archive proof passes at `attempt.4wr4s9`: each extracted crate
checks natively, the consumer runs on Rust 1.99.1 and 1.91 and checks on Wasm,
and all source/archive checksums remain unchanged. The Logging crate SHA-256 is
`5e54050a3d4c4a58c9d390bc95b9bbe11baac68c0f453c35aa0a0d004a08ffd7`.
These are dirty development archives, never clean-release publication receipts.

Strict release-helper Clippy passes. The helper batch has 68 passing tests across
67 initial passes and the corrected archive-transfer-count rerun; preserve the
initial failed log. The new cases cover mandatory Logging receipts, final-upload
rejection/lost-response recovery and missing/corrupt Logging archive refusal
before registry observation. Archive admission and catalog-compiler dispatch
fixtures pass at `/tmp/ic-persistence-package-admission.fnayE1`. Real offline
bootstrap passes both interrupted metadata states at
`/tmp/ic-persistence-bootstrap.KVg7K6` without changing inputs. Formatting,
ShellCheck, graph/pin/boundary, snapshot, link, license and diff checks pass;
both tracked lockfiles remain byte-identical to the parent.

Release-helper tests use substituted Git/registry effects. They never authorize
or prove a real upload. Actual publication still requires a separately selected
clean tagged release and its full release gates.
