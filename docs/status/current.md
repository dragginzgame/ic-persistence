# Current handoff

## Pending Storage 0.65.5

Storage 0.65.5 delivery is authorized; source preparation precedes the standard
release gate, atomic Git delivery and registry publication. Siblings are unchanged.
The [pending notes](../../releases/storage/0.65.md) cover node-buffer improvements,
benchmark bucket controls and the latest Platform adoption. Public Storage APIs,
stored formats, cache budgets and staged root-last publication remain unchanged.

Platform Runtime **0.19.0** is selected for IC0-only Wasm backing. The independent
workspace selects Platform/PocketIC **0.19.0**, Types 0.18.0 and the retained
Testkit 0.79.0 / server 16.1.0 pairing. Its fallible client calls diagnose setup,
installation, clock and cycle failures; rollback checks accept only actual
canister traps. The explicit Platform 0.17 comparison alias remains for ICSS
0.7.2; pinned DFINITY Git `b5d1650` remains the other research control.

Adoption evidence: `target/platform-019-adoption/`. Native client/model tests,
strict native and product-Wasm Clippy, pin/production graph checks, formatting,
and independent default/facade composition on native and Wasm pass. Installed
original-Wasm IO, all five structures, 1,031-allocation upgrade, six directory
faults, custody, Logging and rollback pass in `runtime/`. The five-structure
benchmark smoke passes 216 comparison rows in
`target/storage-comparison/attempt.yeWNhE/` (64 rows, uniform, one round, batches
1/32, ICSS eight-page buckets). No release gate or optimizer matrix was rerun.

The release root lock selects only the Platform changes. Unrelated pending Build
0.74.1, Host 0.74.2, Logging 0.24.3, Metrics 0.27.2 and Wire 0.15.1 updates are
preserved separately for restoration after publication. Independent lock changes
are limited to Platform and required transitive selections/disambiguation. Original
locks/patches are in `target/storage-0655-deploy/` and the adoption evidence.

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

## Earlier pending node-buffer work and bucket study

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

Storage **0.65.4** is published and unyanked: commit `70720ded`, tag
`storage-v0.65.4`, registry/archive SHA-256
`c9ea8338f14d8db0cb2551ff005de50345a0b734e51a96fb67a9c8e9b63414dc`.
Do not repeat release/upload. Evidence: `target/storage-0654-deploy/`,
`target/release-validation/storage-0.65.4.json`, validation `verify.iVFrKc`,
archive `attempt.dCTtqm` and installed original/O3/Os/Oz `attempt.FITlwP`.
Unconfirmed lockfile-writer recovery preserved failed logs, source bytes and
receipts; standard resume succeeded. Keep `storage-0654-unselected-lock-edits`,
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
