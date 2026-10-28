# Current handoff

The released baseline is 0.51.13 at `3acffe19c7d2dc1c2bf70dc03313df95accebd91`.
[Its CI](https://github.com/dragginzgame/ic-persistence/actions/runs/40557536879)
passed all 10 jobs. Formatter #135 and singleton-loader #136 are closed with that
source/native/MSRV/installed evidence. Those runs do not qualify pending edits.

## Pending 0.52.0 backing-custody repair

[#137 — Fail closed when multiple memory runtimes alias one stable backing](https://github.com/dragginzgame/ic-persistence/issues/137)
has a locally implemented and qualified supplier fix. It is uncommitted; package
versions remain 0.51.13 and the index is untouched. The former unpublished
0.51.14 dependency notes are consolidated into the breaking 0.52.0 candidate.
See [current notes](../changelog/0.52.md#0520---unreleased).

Runtime constructors require `MemoryBacking`. Standard VectorMemory and default
canister backings identify aliases automatically; custom/foreign wrappers must
provide a consistent underlying identity. Borrowed, non-Clone, non-Send/non-Sync
hosts remain supported. Process-local claims refuse duplicate runtimes before
backing IO and live through the final detached allocation/collection handle.
Native claims coordinate across threads. Constructors release claims on errors
and ordinary unwind; forgotten owners stay occupied.

A single metadata authority owns the exact expected MGR V1 image and live growth
count. Size, valid IO, opens and growth compare persisted metadata before cached
translations are used. Only successful owned growth projects expected edits;
foreign count, same-count extent, table or unused-entry changes poison the owner.
No foreign image is silently adopted. Invalid IO still refuses before backing
access; diagnostic reports retain their two bounded metadata reads. Existing
MGR/cell/ledger bytes and numeric limits are unchanged; no reset/migration is added.

Raw aliases/unmanaged managers remain prohibited, including over excluded IDs.
Claims require one Memory crate identity and truthful host backing identity;
checks do not provide atomicity against arbitrary raw writes, cross-process
custody or separately linked Memory copies. The linked Canic #527/PR #595,
IcyDB PR #376 and IC Crates PR #7 were reviewed read-only. Their committed-runtime
handoff work is complementary and was not duplicated or edited here.

## Qualification and cost

Evidence is retained at `/tmp/ic-persistence-issue-137/`, with source/lock/Wasm
hashes, initial propagation diagnostics, original/final installed logs and exact
Rust 1.91 Memory/Storage artifacts. Final locked offline qualification passes:

- 68 Memory runtime tests, 5 public runtime integration tests, 17 Storage adapter
  tests and 28 Metrics Stable tests. Six custody cases also execute on Rust 1.91.
- Native all-target Clippy for all affected packages, actual Wasm library/example
  Clippy, and independent harness/test Clippy, with warnings denied; 3 doctests.
- Rust 1.91 builds both installed Wasm fixtures. The original-Wasm runner passes
  Memory IO bounds/traps and upgrade rollback, Storage restoration, and actual
  same-canister custody/composition. Three manager corruption modes trap before
  cached reads and roll back the preceding valid payload write. Runtime drop,
  retained handles, final-lease release and upgrade reopen preserve markers/pages.
- Raw Wasm budgets pass: core 248939/260000, diagnostics 259744/315000,
  admission 252785/264000 and runtime integration 255990/270000 bytes.
- Production singleton/declaration checks, formatting, doc links and diff checks.

The integrity boundary adds one full 34,848-byte metadata read per valid size/IO
check; successful growth verifies before/after. Word comparison reduces the
initial hardened Wasm implementation's sampled 1000 single-byte IO cost from
about 976M to 192M instructions (about 80%), without omitting bytes. This is not
a speed-up over the released unchecked path: added stable IO remains a material
cost. Native comparison uses the optimized host slice comparator, including in
debug tests. Host/application budgets must account for the added checks.

This is local Linux/native and installed PocketIC 16.1.0 qualification. No full
release gate, Binaryen optimization matrix or fresh macOS CI was run. Delivery
and final applicable native acceptance remain separate; no release/publication
or deployment was invoked. The issue remains open for delivery and acceptance.

## Preserved dependency work

The earlier adoption selects Platform 0.7.1 in both catalogs, Tooling 0.69.3
compatible minimums, separately prepared Build/Testkit 0.69.3 CLIs and Metrics
0.16.3. The library lock advanced externally from Tooling 0.69.3 to 0.69.4 during
this task. Its bytes are preserved in `concurrent-Cargo.lock`; cache preparation
and final native lint/tests/MSRV checks use that selection. Both original and
concurrent lock observations remain retained. The independent lock is unchanged
from task entry; source/features, stable-structures 0.7.2 and PocketIC pairing
remain selected. Shared Tooling remains reviewed 0.26.26 at
`5683060a5d8ce79af2fcfedb53b3eb24037fd064`. Earlier dependency proof remains at
`/tmp/ic-persistence-deps-20261028/`; it is not relabelled as new-source evidence.

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
