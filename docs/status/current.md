# Current handoff

Active work adopts published Wire/Derive 0.13.3 in both maintained catalogs,
Testkit Native 0.76.0 and the separately selected Testkit CLI 0.76.0. The native
meter now shares Platform 0.16.0, removing the former native-only 0.15.3 copy.
The incoming root lock already selected Wire 0.13.3; preserve that update.
Types 0.15.1, stable-structures 0.7.2 and the PocketIC 16.1.0 client/server pair
remain unchanged. Existing stored bytes and public contracts do not change;
Wire's new optional fixed-byte-array representation is not selected.

Adoption evidence is retained under `target/adoption-testkit-wire-2026-11-01/`.
Both full locked metadata observations, dependency boundaries, affected-package
and independent-runner Clippy, 44 focused tests, native Rust 1.91 test compilation,
all three Rust 1.91 Wasm fixtures, formatting and documentation checks passed.
Testkit 0.76.0 setup and offline CLI/server admission passed. Original-Wasm
installed IO, corruption/growth refusal, backing custody, Storage reopen, Logging
retention and update/upgrade rollback passed; recorded input hashes verified
unchanged. The unchanged optimizer matrix and native macOS checks were not replayed.
Pending compatible notes are Persistence 0.60.7, Metrics Stable 0.61.5 and
Logging Stable 0.60.5. The maintainer selected supplier-first release and
publication to crates.io and GitHub `dragginzgame/ic-persistence` on `main`.
Release evidence is retained under `target/delivery/testkit-wire-2026-11-01/`;
final receipts and destination readback establish actual delivery.

The preceding Platform 0.16 delivery completed on GitHub and crates.io:
Persistence 0.60.6 (Memory and Storage), Metrics Stable 0.61.4 and Logging
Stable 0.60.4. Each group passed its complete gate. Independent registry
readback matched all retained archive digests and final receipts; GitHub releases
and remote annotated tags matched the release commits, with remote main at
`053697f38853ed93e63032c0c5689f10296654ff`.
See `target/delivery/platform-0.16-2026-11-01/` for delivery evidence.

Native receipt reads share an 8 MiB bounded JSON reader; captured observations
have a 120-second Host Process deadline. Long builds and release gates retain
their execution policy. The preceding focused Platform proof passed original-Wasm
IO rejection, corruption/growth checkpoints, backing custody, Storage reopen,
update/upgrade rollback and Logging retention/reopen/rollback. Input hashes
verified unchanged under
`target/improvements-platform-2026-11-01/installed-original.bWYi12/`.
That proof applies to its recorded selections, not this new adoption.

The common archive boundary was delivered in
[IC Tooling PR #296](https://github.com/dragginzgame/ic-tooling/pull/296).
Persistence retains its archive script until the API is published and adopted.
The validation-reuse investigation measured 11.6 minutes and is recorded in
[the audit](../audits/2026-11-01-release-validation-reuse.md) and
[Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334).
No release proof is reused or gate skipped. The bootstrap launcher now consumes
Shared's source-only observer and rechecks metadata bytes after helper dispatch;
scratch rendering and source/receipt authorization stay with this consumer. See
[#333 — Review a shared read-only observation boundary for grouped Cargo bootstrap](https://github.com/dragginzgame/shared-tooling/issues/333).

Earlier cleanup/delivery evidence remains under `target/complexity-2026-11-01/`,
`target/delivery/cleanup-2026-11-01/` and `target/delivery/upstream-issues/`.
The [earlier full CI matrix](https://github.com/dragginzgame/ic-persistence/actions/runs/41229172386)
at `a6354951fe63e293645c4316d5610971ad750ce5` passed native validation,
installed runtime/upgrade/rollback and compiler-floor checks on Linux and both
macOS architectures, completing #144 and #146. It does not qualify newer inputs.
CI selection #145 retains the exact three-file documentation-only policy and
fails closed to the full matrix; see [CI selection](../ci-selection.md).

The selected Shared snapshot is
`fba200f649d6f1e09c0cb52f04b8c1493c5b0060` (VERSION 0.27.24), with 80 roots and
134 payloads. The real six-state bootstrap fixture and caller cache/error/drift
checks passed locally on Linux; native CI remains separate acceptance. Dirty
sibling rules remain excluded; snapshot files change only through the exporter.
