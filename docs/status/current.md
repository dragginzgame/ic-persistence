# Current handoff

Delivery selected: Persistence 0.62.1, Metrics Stable 0.64.0 and Logging Stable
0.62.1 to crates.io and GitHub `dragginzgame/ic-persistence`, `main`, supplier-first.
Standard full gates, final archive receipts and registry/tag readback establish
completion under `target/delivery/metrics-testkit-wire-2026-11-01/`. The preparation
descriptions below predate this delivery selection. The Metrics detail ledger is
declared as owned release metadata for dated finalization with its root entry.

Published on crates.io and GitHub: Persistence 0.62.0 (Memory/Storage), Metrics
Stable 0.63.0 and Logging Stable 0.62.0. Normal main is at `319dce0`. All three
standard full gates, final archive qualification, registry checksums, Wire/core
requirements and annotated remote tags passed. Delivery evidence remains under
`target/delivery/wire-0.14-2026-11-01/verification.json` and its retained logs.

Current local work adopts published Metrics 0.25.0 in the root catalog/lock.
Default native schema support now shares Wire 0.14 with the adapter. Wire/Derive
0.13 leave the root lock without feature suppression, overrides or conversion
wrappers. The independent qualifier lock and all executable pins are unchanged.
That describes the Metrics-only step; the subsequent Testkit/Wire refresh below
updates the independent Wire lock and selected qualification CLI.
Metrics Stable's public measurement and restoration-error types now carry Metrics
0.25 identity. Hosts must align their direct Metrics requirement before passing
these values to checkpoint/load APIs. Stored rows, arithmetic, bounds and
checkpoint continuity remain unchanged; no reset is needed.

Pending notes select Metrics Stable 0.64.0 for this public dependency identity
change. Memory/Storage and Logging Stable need no release for this adoption.
Package versions, staging, commits, release execution and publication remain
unchanged. Locked metadata/native graph confirm one Wire 0.14 identity. Affected
Clippy, all 29 adapter tests, Rust 1.91 native/Wasm and graph/pin/format/docs checks
passed. Focused evidence is retained under `target/metrics-0.25-adoption/`.
The previous native Wire 0.13 caveat is retired by this supplier adoption;
published release notes remain historical evidence and are not rewritten.

The subsequent local refresh selects Testkit Native 0.78.0 and Wire/Derive 0.14.1
in the root lock, with Wire/Derive 0.14.1 in the independent qualifier lock.
Compatible Wire 0.14.0 catalog requirements remain unchanged. The Testkit CLI pin
is now 0.78.0; explicit setup reused its admitted published installation and
authenticated the unchanged PocketIC server 16.1.0 selection. No Platform/runtime
owner, product limits or stored format changed. The pending Metrics adoption is
preserved. Compatible notes select Persistence 0.62.1 and Logging Stable 0.62.1;
Metrics Stable retains its pending 0.64.0 identity cut. Current package versions,
staging, commits and publication remain unchanged. Evidence is retained under
`target/testkit-0.78-wire-0.14.1/`.

Both graph/Clippy selections, pins/production admission, 33 codec tests and nine
memory-accounting tests passed. Rust 1.91 native/Wasm and formatting/docs checks
passed. The new CLI passed installed Linux Logging retention/reopen and
update/upgrade rollback in `target/qualification/runtime/attempt.KvhNVn/`;
post-run input hashes verified unchanged. The earlier sandbox attempt
`attempt.3bCN7R/` refused localhost binding before runtime checks and is retained.
Full release gates, other installed suites and native macOS were not replayed.

The published Wire 0.14 hard cut changes public codec-error identities while
preserving durable bytes and admission. Its focused proof passed 33 codec and
three qualifier tests, selected Clippy/graph checks and Rust 1.91 native/Wasm
checks. Evidence remains under `target/wire-0.14-adoption/`. The release also
includes Testkit Native 0.77.0, sharing Platform 0.17.0 for metered-memory helpers;
its focused proof and saved incoming lock are under
`target/testkit-native-0.77-adoption/`.

The preceding Platform 0.17/Wire 0.13.5 releases remain retained under
`target/delivery/platform-wire-2026-11-01/`. Stable-structures 0.7.2 is unchanged.
The native qualifier uses owned `ic-platform-pocket-ic` 0.17.0 with server 16.1.0;
Logging's Wasm fixture uses Runtime/macros/guard 0.17.1 and Types 0.16.0.
That proof selected Testkit CLI 0.76.0. Installed Linux IO, Storage, custody, Logging
retention and update/upgrade rollback passed with unchanged post-run input hashes
under `target/platform-0.17-adoption/`. Native macOS and installed optimizer
variants retain their separate acceptance.

The earlier public API hard cut uses explicit Storage `create`/`load`/`reset`, one
bound-enforcing `StableCell`, parsed `StableKey` runtime lookup/open and nonzero
bucket configuration. See [the migration](../migration/0.61-public-api.md).
Earlier focused/installed proof remains under `target/hard-cut-api-2026-11-01/`
and `target/complexity-followup-2026-11-01/`.

The first hard-cut releases used retained isolated delivery checkout
`target/delivery/hard-cut-observability-2026-11-01/checkout/` to preserve incoming
lock work. Its completed receipts/readback own delivery; the original failed
Persistence 0.61.0 plan is superseded evidence and must not be resumed. Preserve
autostash backups, failed attempts, receipts and content-addressed archives.

Build owns extracted archive graph admission/input custody; the consumer owns
its library/native/MSRV/Wasm roster. Shared's grouped bootstrap observer is
adopted from [PR #336](https://github.com/dragginzgame/shared-tooling/pull/336).
The merged destination-adoption check covers independent graphs for
[Shared #335](https://github.com/dragginzgame/shared-tooling/issues/335).
Opt-in ShellCheck reuse retains complete gates and fresh group admissions.
PR #150 admits macOS tool bytes within 64 MiB and preserves failure logs/status.
Native CI remains separate acceptance for
[Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334); retain
failed run 41314835837 and existing local proof. See
[adoption](../shared-tooling-adoption.md#exact-input-shellcheck-reuse).

The reviewed Shared snapshot remains
`9275bf0de39ed941d81c47bab97eef7f6fc38df2` (VERSION 0.27.24), with 81 roots and
136 payloads. Never patch vendored files or adopt dirty sibling rules.
