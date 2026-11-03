# Current handoff

## Delivered Storage 0.65.5

Storage **0.65.5** is published and unyanked. Commit
`127031ee7013a0ab3c5c9de38d5e1722c529280d` and annotated tag `storage-v0.65.5`
were atomically pushed to `origin/main`. The downloaded registry archive and
sparse index match the qualified SHA-256:
`4a5b1ea9a7b8f61e0ea22b257be5282795bd24ba77ab3d730afd02a46462b9aa`.
Do not repeat release/upload. Only Storage was released; siblings are unchanged.
The [release notes](../../releases/storage/0.65.md) cover node-buffer improvements,
benchmark bucket controls and the latest Platform adoption. Public Storage APIs,
stored formats, cache budgets and staged root-last publication remain unchanged.

Platform Runtime **0.19.0** is selected for IC0-only Wasm backing. The independent
workspace selects Platform/PocketIC **0.19.0**, Types 0.18.0 and the retained
Testkit 0.79.0 / server 16.1.0 pairing. Its fallible client calls diagnose setup,
installation, clock and cycle failures; rollback checks accept only actual
canister traps. The explicit Platform 0.17 comparison alias remains for ICSS
0.7.2; pinned DFINITY Git `b5d1650` remains the other research control.

Delivery evidence: `target/storage-0655-deploy/`. The exact release checkout is
retained at `checkout/`; its `target/` owns final receipts and archives. Full gate
`verify.ze8HOu` passed in 247.558 seconds; final archive integration is
`package-qualification/attempt.H6blMA`. Installed original/O3/Os/Oz IO, all five
structures, 1,031-allocation upgrade, six directory faults, custody, Logging and
rollback pass in `qualification/runtime/attempt.a4RQq4`. Publication and independent
registry/archive verification are retained alongside the checkout.

Earlier adoption checks remain in `target/platform-019-adoption/`, including
native/Wasm default/facade composition. The five-structure smoke passed 216 rows
in `target/storage-comparison/attempt.yeWNhE/` (64 rows, uniform, batches 1/32,
one round, ICSS eight-page buckets). No whole-endpoint speedup is claimed.

The release root lock selects only the Platform changes. Unrelated pending Build
0.74.1, Host 0.74.2, Logging 0.24.3, Metrics 0.27.2 and Wire 0.15.1 updates are
restored in the main checkout, byte-identical except Storage's release version.
Keep the `storage-0655-unselected-lock-edits` stash and saved locks/patches.
The first gate passed its tests but refused admission after an unconfirmed writer
restored those lock updates. An isolated retry with shared build output hit
trybuild path mismatches. A clean checkout with independent artifacts passed the
unchanged full gate. Both failed logs remain; no checks or assertions were bypassed.

## Local memory audit

The [consumer audit](../local-memory-audit.md) records source inventories and
compatibility limits. Canic core declares 27 base plus five optional allocations;
control-plane additions depend on role. Blob declares 14 per installation;
Relay uses three per channel plus component stores; IcyDB generates `3 + 4J`.
These are named allocations, not ICSS 8 MiB buckets or deployed usage counts.
Sibling working trees were inspected read-only; many contain pending migrations.

Priority follow-ups: IcyDB's schema control map at commit page 257 forces at least
16.125 MiB when initialized; consider a separate small control allocation only
with explicit layout migration/fresh-image qualification. Canic's live receipt
reservation still assumes ICSS 0.7.2 geometry; requalify its bound against Storage.
No demonstrated capacity failure or quantified Canic saving is claimed. Use
existing per-key/global diagnostics before consolidating any further stores.

## Node-buffer work and bucket study

B-tree nodes now encode into their final checksummed frame; membership retains
node/index without value copies, and `get` decodes borrowed inline bytes.
External payloads retain their owned checked buffers. Evidence in
`target/storage-node-buffers/`: 54 adapter tests including 24,000 model operations,
frame compatibility and codec ownership/failure checks, strict native/Wasm Clippy,
and installed original-Wasm lifecycle qualification (`runtime-final/`).

Matched baseline/final runs validated 3,744 calls with matching results/pages and
DFINITY timings. Final runs: `attempt.GAjrJo` (4,096 shuffled integer rows, three
rounds, uniform/hot-16/Zipf, Storage 128 KiB cache, ICSS Git 32 slots) and
`attempt.ZCtVSr` (64 byte-map rows, three rounds, uniform, no caches, ICSS 0.7.2)
under `target/storage-comparison/`. Uniform batch-32 integer Get improves 2.48%,
Insert 2.04%, single Get only 0.31%. **4 KiB batch Scan regresses 12.74% and the
complete 4 KiB trace rises 0.15%.** Cause remains unverified; profile before
claiming across-workload improvement. Cache limits are not equal heap budgets.

`BENCH_UPSTREAM_BUCKET_PAGES` / `--upstream-bucket-pages` is a nonzero-u16 ICSS
comparison setting, default 128, recorded in reports for all five structures.
Nine matched 1/8/128-page runs validated 6,720 calls, with identical Wasm per
backend/workload, caches off and matching results. Evidence:
`target/storage-bucket-study/{results.json,results.csv,analysis.log}`.
Integer ICSS pages: 4/9/129 versus Storage 6; all 96 integer instruction rows
match at 8 and 128 pages. 4 KiB byte-map batch Get: ICSS 13/17/129 pages versus
Storage 28 after mutation history; instructions 2,156,312/2,104,898/2,086,854
versus Storage 2,745,692. Largest measured ICSS increase over 128-page buckets:
6.47% at one page, 1.43% at eight. These are selected collection workloads, not
whole endpoints, a production-default selection or a universal superiority claim.

## Delivered releases, recovery and retained boundaries

Storage 0.65.4 and 0.65.3 remain published. Earlier unconfirmed lockfile-writer
recovery preserved failed logs, source bytes and receipts; standard resume
succeeded. Keep `storage-0654-unselected-lock-edits`,
`storage-0653-unselected-lock-edits` and `storage-0653-hotspot-integration` stashes.
Storage 0.65.3 and Metrics Storage 0.66.0 remain published. Logging Storage stays
0.63.0 with 0.64.0 notes pending. Full prior recovery facts/evidence are in the
[0.65.4 handoff](https://github.com/dragginzgame/ic-storage/blob/b9010ba/docs/status/current.md).

Storage owns allocation, custody and five collection forms; opaque IDs, host
lifecycle/framing and explicit fresh/reset backing remain. Map/Set/regions retain
`ICMAP002`; earlier images require explicit rebuild, never automatic reset or
legacy decoding. Native failures poison ownership; IC traps use rollback.
Shared Tooling 0.27.27 (`651f5f7`) and Rust 1.99.1 remain selected. Native
production has no Runtime; default Wasm has no executor, CDK, guard, Types or
Candid. No historical tool caches were removed. Cell checksum throughput and
external-payload scan regression remain profiling follow-ups.
