# Current handoff

## Delivered Storage 0.65.3

Storage **0.65.3** is published and unyanked. Release commit
`8c749ca2cd3859dfe4545f1792d7e8d2ff460002` and annotated tag `storage-v0.65.3`
were atomically pushed to `origin/main`. The downloaded registry archive, fresh
sparse-index record and final receipt agree on SHA-256:
`74ee2e0c2e95769a67e9602184470f8258130c3935e0f52aa74471fc9104e28f`.
Do not repeat release/upload. Evidence: `target/storage-0653-deploy/` and
`target/release-validation/storage-0.65.3.json`.

This compatible patch delivers cache capacity/routing retention, lazy mutation
copies and corrected comparison phases. Public APIs and formats remain unchanged;
caching remains disabled initially. See [release notes](../../releases/storage/0.65.md).
It preserves 0.65.2's Runtime ^0.18.1 IC0-only Wasm backing. Native production has
no Runtime dependency; default Wasm has no executor, CDK, guard, Types or Candid.
The [0.65.2 qualification record](../ic0-backing-qualification.md) retains its
registry-consumer, publication and isolated-checkout evidence. Relay and sibling
checkouts were not edited; [Relay #21](https://github.com/dragginzgame/ic-relay/issues/21)
remains downstream adoption.

## Qualification and checkout custody

Implementation commit `1cad31a` passed the complete release gate in 214.365 seconds
(`target/release-validation/attempts/verify.WzEQr0`). Final archive integration
passed in `target/package-qualification/attempt.TQ4gVq/` after the release commit.
Installed IO, growth/refusal, bounds, reopen, upgrades, rollback, directory faults,
backing custody, all five collection forms and Logging passed for original,
O3, Os and Oz Wasm with Testkit 0.79.0 / PocketIC 16.1.0 in
`target/qualification/runtime/attempt.iSSaeB/` against the exact release.
Linux-local qualification does not establish native macOS execution.

The first attempt stopped before commit because the root lockfile acquired
unselected dependency upgrades after its qualified checksum was recorded. The
exact source-derived projection matched that receipt; restoring it allowed the
same saved release to resume without weakening admission. Those unselected edits
are preserved in stash `storage-0653-unselected-lock-edits` and restored as pending
root lock changes after publication. Their origin is unconfirmed; they were not
included in 0.65.3. Original/changed lock bytes, failed log and receipt remain in
the deployment directory. The earlier hotspot integration stash is also retained.

## Independent groups and retained contracts

Metrics Storage **0.66.0** remains published (tag `metrics-storage-v0.66.0`, commit
`64777ae7a5ca916ecfcf7db94342b22008dc02c9`). Logging Storage **0.63.0** remains its
package version with **0.64.0** notes pending. Neither group was released or
published by this delivery. Storage's release updated only its own identity and
required local catalog/lock references; adapter versions and notes are unchanged.

Storage owns allocation, backing custody and Cell, Map, Set, Vector and map
regions. IDs remain opaque; hosts own grants, lifecycle, migration framing and
explicit reset/fresh backing. Map/Set/regions retain `ICMAP002` from 0.65.0.
Earlier AVL images require explicit rebuild/reset. No compatibility decoder or
automatic reset exists. Native backing failures poison the owner; IC traps retain
host rollback. See [SAFETY](../../SAFETY.md) and [allocation](../current-ledger.md).

Wire 0.15 owns current codec errors; Metrics Storage uses Metrics 0.27. Logging
checkpoints retain discriminator 3 and an explicit retained-text ceiling. Hosts
retire earlier rows explicitly. Storage 0.65.1's diagnostics, free-block reuse,
optional node caching, bounded scans and vector/extent IO improvements remain.

## Validation and benchmark evidence

Pre-integration evidence is in `target/storage-hotspots/`: 87 library tests, four public
runtime tests, nine benchmark model tests, 24,000 generated model operations,
strict native/product-Wasm Clippy and Wasm test compilation pass. Installed
original-Wasm IO, upgrade, rollback, directory faults, backing custody, all five
structures and Logging pass in `runtime/`. The full-matrix smoke passes 4,408
calls: 64 rows, uniform keys, two rounds, caches disabled, upstream 0.7.2.
An overly broad Wasm Clippy selection hit the existing helper example's
`missing_const_for_fn`; product Wasm linting and full Wasm test compilation pass.
An earlier lock refusal during concurrent Platform edits is also retained.
The integrated release gate and optimizer matrix are recorded above; the
comparison itself retains its original isolated dependency graph.

The comparison covers 28 cases / 46 type-size combinations across all five
structures. The published control uses upstream 0.7.2 through Platform 0.17;
an optional comparison-only Git control pins upstream `b5d1650`. Production has
no upstream dependency. See the [benchmark methodology](../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md).

Controlled evidence lives under
`target/storage-hotspots/qualification/target/storage-comparison/`:
`attempt.lrdVtZ` is unchanged 0.65.1 with corrected phases, `attempt.oFKN07` adds
cache changes, and `attempt.gVYkXo` adds lazy mutation copies. Each passes 576
calls and final models on 4,096 shuffled integer-map rows, one population seed,
three rounds, uniform/hot-16/Zipf keys and batches 1/32. Storage uses 128 KiB;
upstream Git `b5d1650` uses 32 slots. These are not equal heap budgets. The
isolated checkout at `a82de9b` holds dependency selections fixed. The complete
default matrix is `attempt.42gavZ`.

Uniform single-call instructions before/after: Get 174,176/173,911, above-range
miss 177,502/2,990, Replace 211,053/185,423, Remove 416,728/349,677 and true
RemoveMiss 217,403/90,094. Uniform batch-32 Remove falls 3,958,383→1,828,227;
Replace 2,379,548→1,338,588; RemoveMiss 1,098,962→228,281. Upstream single Get
still costs 95,965. These are collection intervals, not whole endpoints.
Historical runs remain under `target/storage-comparison/`; their RemoveMiss
and grow/shrink labels did not establish the intended phase preconditions.
New traces enforce those preconditions and clean up inserted keys each round.
No general superiority, GiB qualification or 2× whole-call guarantee exists.

## Tooling and history

Shared Tooling 0.27.27 (`651f5f7`) remains the reviewed executable snapshot;
AGENTS' revision-bound policy remains authoritative. Rust 1.99.1 is the support
floor and pinned compiler. The release selected Build/metadata CLI 0.74.0, Host 0.74.1, Testkit CLI
0.79.0 and server 16.1.0 are unchanged. Both locks select Runtime 0.18.1 where
applicable; the independent qualifier's canonical guard is 0.18.1 and its native
client remains 0.18.0. No historical tool caches were removed.

Prior publication, comparison measurements and retained artifact paths remain in
the [0.65.1 handoff](https://github.com/dragginzgame/ic-storage/blob/a82de9b8682ddfff62a1cdaef09c189f229fd861/docs/status/current.md)
and [comparison provenance](../storage/upstreams.md). Benchmark results remain
mixed; large-record slack, directory snapshots and unreclaimed tails remain
costs. This release makes no superiority, GiB-scale or whole-call speed claim.
