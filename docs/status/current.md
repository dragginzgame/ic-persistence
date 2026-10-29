# Current handoff

Released source is 0.54.1 at
`447a9bb272ffa4551fe6c6ad7f7d1211121f99f0`. Pending notes now select breaking
0.55.0 for Metrics Stable's Wire storage cut and the earlier unpublished Logging
codec adoption. Package versions remain 0.54.1. No staging, commit, push, release,
installed-runtime execution or actual stable-data deletion was performed.

## Accepted storage cut

The maintainer explicitly approved a hard cut, no legacy decoders, and retirement
of all old Metrics data. Hosts must clear the metrics map with their old release
before upgrading or supply a known fresh governed map, and advance reset epochs.
Never reset Memory's allocator or unrelated collections to clear metrics.
[Upgrade disposition](../../crates/ic-metrics-stable/README.md#upgrade-disposition)
is canonical. Old rows are refused without replacement; no automatic reset or
migration engine is introduced.

Metrics now uses `ICMW` plus a bounded IC Wire native Serde/Postcard envelope,
with a borrowed state byte string containing a separately bounded Wire payload.
This preserves explicit removal of invalid numeric state after envelope admission.
Private Serde state records feed Metrics' canonical arithmetic constructors, so
numeric failures retain typed errors. Identity/configuration, append continuity,
static histogram admission and unchanged zero-write saves stay in Persistence.
The complete record ceiling is 6,144 bytes; histograms still admit 256 finite
bounds. Each codec operation allows 1 MiB conservative allocation charges,
16,384 nodes and depth 16. These are admission allowances, not measured heap use.

Logging's earlier Wire implementation remains in the working batch: borrowed
checkpoint fields are checked before canonical `Log::restore`, and the format
uses `ICL` plus discriminator 2. Prior experimental Logging bytes require explicit
host retirement or rewrite using old tooling. Logging remains unpublished; the
three-package publication roster and Memory/Storage formats/custody are unchanged.

The root already selected Wire 0.7.21 without default features and Serde. Metrics
now inherits those selections; no external package versions changed. Neither the
independent qualification graph nor compiler/tool/server selections changed.

## Focused evidence

Metrics' 29 unit tests and three doctests pass with the locked offline graph on
Rust 1.99.1. Tests cover frozen current Wire bytes, populated reopen, wide numeric
state, maximum/zero histogram shapes, malformed/truncated/trailing records,
collection/record limits, typed arithmetic refusal, canonical mutation admission,
continuity and refused/unchanged zero-write checkpoints. Initial maximum histogram
cases exposed insufficient traversal allowances; the corrected budgets pass.

Strict native all-target and Wasm library Clippy pass for Metrics. Actual Rust
1.91 Wasm library compilation passes. Dependency pins, production graph and doc
links pass; formatting and diff checks pass. Logs are retained under
`/tmp/metrics-wire-*`, including failed development checks. No performance delta,
full release, macOS or installed Wire-format upgrade qualification is claimed.

Earlier Logging evidence remains bound to its source: nine tests, two doctests,
native/Wasm strict Clippy and Rust 1.91 compilation. The earlier frozen Metrics
V1 results qualify only the now-retired layout. Historical evidence is never
relabelled as acceptance of the new format. Prior handoffs/evidence are retained
at `/tmp/ic-persistence-before-metrics-wire-handoff.md` and
`/tmp/ic-persistence-logging-wire.AFMlbH/`.

## Sibling codec follow-ups

The requested scan is tracked in existing
[Canic #381 — Stable record encoding](https://github.com/dragginzgame/canic/issues/381),
[Auth #109 — Replace stored-record codec](https://github.com/dragginzgame/ic-auth/issues/109),
[IcyDB #592 — Investigate schema artifact codecs](https://github.com/dragginzgame/icydb/issues/592),
and [Profiler #5 — Investigate process transport](https://github.com/dragginzgame/ic-profiler/issues/5).
No sibling source was edited. These owners retain format, retained-state and
integration decisions; no speedup or implementation is claimed there.
