# Current handoff

## Delivered Storage 0.65.1

Storage **0.65.1** is published and unyanked. Release commit
`055cf5a48f7aad0449bced8d2d46fd953f1bbf9a` and annotated tag `storage-v0.65.1`
were atomically pushed to `origin/main`. Fresh sparse-index and downloaded-archive
checks match the final receipt checksum:
`5af04fdb60433b12387baf8f3529c7bc8be63b6d492da43d70e80f9921751423`.
Evidence is in `target/storage-0651-deploy/` and
`target/release-validation/storage-0.65.1.json`. Do not repeat release/upload.

This compatible patch delivers checked allocation diagnostics, simpler directory
preparation, free-block splitting and conservative live-record shrink reuse,
optional byte-budgeted map node caching, single-buffer routing views, fewer
ancestor writes, bounded key-only ranges, vector IO/buffer reuse, extent
translation caching and bounded-scan fixes. Caching starts disabled. Stored
formats are unchanged from 0.65.0; no consumer migration or sibling release ran.
See [release notes](../../releases/storage/0.65.md) and
[upstream comparison provenance](../storage/upstreams.md).

The first full gate stopped at Wasm test compilation because an IO-count test
used native-only `MeteredMemory`. Source commit `46b41b6` gates that test to native;
production code is unchanged by the repair. The failed log remains at
`target/release-validation/attempts/verify.1nBWcF`. The corrected complete gate
passed in 192 seconds (`verify.wcDBds`), followed by prepared/final archive
qualification. Final package proof is `target/package-qualification/attempt.y2Xkok/`.
The earlier implementation source is `14e9187`; all accepted source work is committed.
No subsequent release is selected. Build artifacts and failed evidence remain.

## Independent groups and current contracts

Metrics Storage **0.66.0** remains published (tag `metrics-storage-v0.66.0`, commit
`64777ae7a5ca916ecfcf7db94342b22008dc02c9`). Its registry proof remains in
`target/metrics-storage-066-deploy/` and its group receipt. Logging Storage
**0.63.0** remains the package version with **0.64.0** notes pending; it was not
released by this deployment. Adapter versions remain independent of core patches.

Storage owns allocation, backing custody and Cell, Map, Set, Vector and map
regions. There is no Memory package or compatibility facade. IDs are opaque;
hosts own grants, lifecycle, migration framing and explicit reset/fresh backing.
Map/Set/regions use the `ICMAP002` B-tree introduced in 0.65.0. Earlier AVL images
require explicit rebuild/reset; no legacy decoder or automatic reset exists.
Small values stay inline; external values are separately checksummed. Membership
and key-only scans do not audit external value integrity. Native backing failures
poison the shared owner; IC traps retain host rollback. See [SAFETY](../../SAFETY.md)
and [the allocation contract](../current-ledger.md).

Wire 0.15 owns current codec errors. Metrics Storage uses Metrics 0.27. Logging
Storage's maintained checkpoint contract uses discriminator 3 and a retained-text
ceiling; hosts retire earlier rows explicitly. No sibling edits are included.
Storage #152 is resolved by the published native dependency boundary. Wasm still
selects Runtime's execution feature; narrowing that integration remains separate
from this release. Track [Platform #135](https://github.com/dragginzgame/ic-platform/issues/135)
and downstream [Relay #21](https://github.com/dragginzgame/ic-relay/issues/21).

## Validation and benchmark evidence

`target/upstream-improvements/` retains 85 passing library tests, four public
runtime tests, 24,000 generated model operations, strict native/Wasm checks and
production graph checks. Installed original-Wasm IO, upgrade, rollback, directory
faults, backing custody, all five structures and Logging pass in
`runtime-final-qualified/`. The earlier artifact-path failure is retained in
`runtime-final/`. These installed tests precede the native-only test annotation
and release metadata update; the full release gate covers the final source.
Linux evidence does not establish native macOS or an optimizer runtime matrix.

The comparison covers 28 cases / 46 type-size combinations across all five
structures. The published control uses upstream 0.7.2 through Platform 0.17;
an optional comparison-only Git control pins upstream `b5d1650`. Production has
no upstream dependency. See the [benchmark methodology](../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md).

Matched pre-release HEAD comparisons are in `target/storage-comparison/attempt.ey0896/`
(Storage cache 128 KiB, upstream 32 slots) and `attempt.PkfT47/` (both disabled).
Each passes 4,212 calls and final model checks with identical Wasm hashes: 4,096
rows, integer maps plus six byte-value sizes, one shuffled population seed,
three mutation rounds, uniform/hot-16/Zipf keys and batches 1/32. These are
collection intervals, not whole endpoints; cache settings are not equal heap budgets.
Storage's single-read cache gains are 1.09× uniform, 30.02× hot-16 and 2.06× Zipf.

Results are mixed. With caches enabled, uniform single-operation Storage/upstream
ratios are 1.8125 for integer Get, 1.1092 for Replace, 0.4806 for a one-entry
bounded scan and 0.3987 for a 4 KiB-value Get. A 4 KiB-value missing-key trace costs
176,082 versus 2,502 instructions. Integer maps reserve 6 versus 129 stable pages,
but 4 KiB values reserve 519 versus 385. Reservation is not live payload. Cache
placement, large-record slack, directory snapshots and unreclaimed tails remain
costs. No general superiority, GiB qualification or 2× whole-call guarantee exists.
The full-matrix 64-row draft smoke is `attempt.i6I874/` (2,080 calls); it predates
the conservative shrink adjustment. Other historical runs remain retained.

## Tooling and retained history

Shared Tooling 0.27.27 (`651f5f7`) remains the reviewed executable snapshot;
the AGENTS policy reference remains authoritative. Rust 1.99.1 is the support
floor and pinned compiler. Build/metadata CLI 0.74.0, Host 0.74.1, Testkit CLI 0.79
and server 16.1.0 are unchanged. Root locks retain Runtime 0.18.1, Metrics 0.27.1
and Logging 0.24.2; the independent qualifier retains its 0.18.0 client graph.
Unused Testkit Native was removed; Storage owns memory metering under test-support.
See [adoption boundaries](../shared-tooling-adoption.md) for exact selections.

Earlier publication and recovery narratives are retained in the
[pre-release handoff](https://github.com/dragginzgame/ic-storage/blob/14e9187/docs/status/current.md).
Existing `target/storage-065-deploy/`, `target/metrics-storage-066-deploy/`,
`target/storage-audit-improvements/`, `target/storage-scan-audit/`, supplier-adoption
and release-validation evidence remains intact. Intentional historical-note
retirement is preserved through the ledgers' revision-bound links. Never remove
historical caches, failed attempts, receipts or release archives implicitly.
