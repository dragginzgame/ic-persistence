# Current handoff

The checkout is based on 0.56.1 `d4208ce`; the ten requested cleanup changes are
uncommitted, with pending notes for 0.56.2. No commit, release, publication or
push ran. Selected dependency versions remain unchanged. The root lock adds only
Storage's dev dependency edges to Wire and Serde; the independent qualification
manifest and lock are unchanged.

## Ten completed complexity reductions

1. Memory and Storage fixtures share bounded Wire request/response schemas with
   the host runner, including method identities and lifecycle arguments.
2. Memory scenarios and IO measurements live in their own runner module; suite
   setup/dispatch and the existing CLI modes remain in the entry point.
3. Archive SHA-256 identities stay typed until receipt serialization.
4. Publication reuses admitted artifact paths while preserving per-effect source
   identity checks, tag checks and registry reconciliation.
5. A standalone Perl renderer owns disposable packaging manifests; generated
   bytes match the former inline renderer, including quoted paths.
6. Recovery fixtures use shared byte snapshots without removing assertions.
7. Policy, manager-ID and page-size code uses existing owned numeric limits.
8. Identical namespace-prefix refusals share one branch.
9. Generation staging checks its successor directly with the domain error.
10. Declaration-claim validation owns its single-caller slot-reuse check.

Public library APIs, error ordering, custody, persisted collection/ledger bytes
and release safeguards remain unchanged. Only the private qualification packet
format is cut over; fixture and runner must be rebuilt together. No legacy
packet reader is included.

## Focused evidence

Native logs and renderer parity evidence are under
`/tmp/ic-persistence-round3.9AXv4N/`. All 69 release-helper tests pass; Memory
passes 187 library tests (eight explicit ignores), Storage 17 and the independent
runner three (two explicit ignores). Strict affected native/host/Wasm Clippy,
Rust 1.91 Wasm checking, pin/production-graph checks and archive-admission fixtures
pass. The extracted renderer produces byte-identical workspace, override and
consumer manifests. Final formatting, documentation links and diff checks pass.

Linux installed original-Wasm qualification passes through Testkit CLI 0.70.0
and PocketIC 16.1.0. The run covers refused Wire frames, invalid IO, upgrade and
update rollback, Storage reopen, manager faults, custody and Logging watermarks.
Exact launch/source/tool/binary hashes and logs are retained under
`target/qualification/runtime/wire-fixtures.0dwbpz/`; source hashes match after
the run. See [the installed record](../runtime-io-qualification.md#shared-wire-fixtures-installed-qualification--2026-10-30).
Fresh instruction/cycle CSVs belong to these Wire builds. Earlier raw-packet
measurements are not relabelled; no performance gain is inferred. Optimizer
variants, macOS and full release gates were not rerun.

## Retained-data boundary

The [ledger contract](../current-ledger.md) remains Wire-only with `ICML` framing
and `ICMWIRE1` protected generations. Earlier CBOR images require host-owned
fresh/reset installation or a separately reviewed transition. No legacy decoder,
fallback or automatic reset is included; never clear only governance while
leaving application collection bytes behind.
