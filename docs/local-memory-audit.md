# Local consumer memory audit — 2026-11-03

This is a source audit of local working trees, not a measurement of deployed
canisters. Siblings were read-only. Many have pending migrations; their current
declarations must not be confused with their published versions or selected
lockfiles. No consumer layout, public API or retained image was changed.

## What a memory count means

The inspected current consumers use Storage-issued named allocations. These
share a canister's physical stable backing; they are not separate Wasm heaps
or ICSS MemoryManager buckets. Storage records new names with no data extents
and grows by the requested number of 64 KiB pages. Directory metadata also
occupies real pages. See [directory construction](../crates/ic-storage/src/runtime/directory_model.rs)
and [physical accounting](../crates/ic-storage/src/runtime/directory_model/persistence.rs).

A declaration can remain at zero data pages. Opening/initializing a collection
and writing application data are separate events. Consequently, multiplying
these inventories by ICSS's default 128-page (8 MiB) bucket is incorrect.
Likewise, summing components without considering canister roles double-counts
mutually exclusive deployments.

## Verified inventories

| Component | Named allocation inventory | Implementation and actual use |
| --- | --- | --- |
| Canic core | 27 unconditional, plus up to 5 feature-selected | [Key/request factory](../../canic/crates/canic-core/src/memory/keys/mod.rs): three optional auth-state keys and two sharding keys. [Memory bootstrap](../../canic/crates/canic-core/src/memory/mod.rs) admits the complete inventory; [intent stores](../../canic/crates/canic-core/src/storage/stable/intent.rs) initialize lazily. |
| Canic control plane | Root adds 21; fleet coordinator adds 3; Wasm store adds 6 | [Role-gated keys](../../canic/crates/canic-control-plane/src/memory/keys/mod.rs): 17 root-only, 3 fleet-only, 2 Wasm-only, 4 shared root/Wasm. [Composition](../../canic/crates/canic-control-plane/src/memory/mod.rs) adds these to core. Combined feature builds use the union, not the sum. |
| Canic metrics | 0 or 1 additional allocation per role | [Provider host](../../canic/crates/canic-core/src/ops/runtime/metric_providers/host.rs) declares one shared metrics map only when a compiled role provider needs stable storage. It is not one memory per metric. |
| Blob Storage | 14 per installation | [Thirteen service stores](../../ic-blob-storage/crates/ic-blob-storage/src/ops/service/stores/grants/mod.rs): ten upload, two funding, one gateway; [installation](../../ic-blob-storage/crates/ic-blob-storage/src/ops/service/installation/mod.rs) adds configuration. [Canic adapter](../../canic/crates/canic-blob-service/src/ops/memory/mod.rs) composes these with its framework inventory. |
| Relay | 3 per channel; +1 per responder, dispatch, mailbox or durable allocator instance | [Channel](../../ic-relay/crates/ic-relay/src/channel/mod.rs) creates stream/outbox/inbox maps; [responder](../../ic-relay/crates/ic-relay/src/responder/mod.rs), [dispatch](../../ic-relay/crates/ic-relay/src/dispatch/mod.rs), [mailbox](../../ic-relay/crates/ic-relay/src/mailbox/mod.rs), and [resources](../../ic-relay/crates/ic-relay/src/resources/mod.rs) own the additions. The actual [two-peer qualification host](../../ic-relay/crates/ic-relay-persistence-probe/examples/mailbox_peers/installed/storage.rs) declares 18: four shared plus seven per peer. This is a fixture count, not a universal Relay count. |
| IcyDB | `3 + 4 × journaled stores` per generated canister | [Generator](../../icydb/crates/icydb-model/src/build/actor/db/store.rs) adds commit, integrity progress and startup, then data/index/schema/journal per journaled store. Heap stores add none. [Facade fixture](../../icydb/testing/model-facade-only/src/design/mod.rs) has one journaled store (7); [logical-memory fixture](../../icydb/testing/logical-memory/src/schema.rs) has two (11) unless its retiring-store omission feature is enabled. |
| Metrics Storage / Logging Storage | One host-supplied map per store instance | [Metrics](../crates/ic-metrics-storage/src/lib.rs) and [Logging](../crates/ic-logging-storage/src/lib.rs) wrap supplied governed maps; they do not construct independent managers. Hosts decide how many instances exist. |
| Auth / Scheduling | No fixed production allocation count imposed by the library | Their qualification consumers use explicit host memories; those fixtures are not allocations automatically created by linking the libraries. |

The scan also covered ic-backup, ic-crates, ic-observability, ic-profiler,
ic-query, ic-replica-watch, ic-tooling, ic-wire and plex. No additional active
ICSS manager/bucket factory was found in the scanned production sources.
This does not establish that every host has no durable state. Plex has no root
Rust workspace. Toko and Toko Miner were excluded under their scope restrictions.

The local ic-crates registry declares one journaled store, but its root lock
still selects Storage 0.56.4 while pending catalogs select newer Canic/IcyDB.
Do not present the current IcyDB generator's seven-allocation result as a
qualified ic-crates build. Canic, Auth, Blob, Relay, Scheduling and IcyDB root
locks select Storage 0.65.4. Their current working trees may contain later work.

## 1. Remove IcyDB's forced commit-buffer prefix on a future layout change

**Verified:** [schema control](../../icydb/crates/icydb-core/src/db/schema/control_store.rs)
places its single shared control map at page `MAX_COMMIT_BYTES / 65_536 + 1`,
which is 257. [Commit marker limits](../../icydb/crates/icydb-core/src/db/commit/marker.rs)
set `MAX_COMMIT_BYTES` to 16 MiB. `control_memory()` returns the commit allocation.
Initializing the map reserves space at that offset; Storage's
[region growth](../crates/ic-storage/src/adapter/region.rs) grows the underlying
allocation through the entire prefix. It is not a sparse virtual reservation.

**Consequence derived from the code:** once this map initializes, the commit
allocation must contain at least 258 pages (16.125 MiB), even if its commit
payload and control records are tiny. Actual records can require more pages.
The region's much larger end bound is only a limit, not an eager allocation.
[Schema application](../../icydb/crates/icydb-core/src/db/schema/application_store.rs)
and [live checkpoints](../../icydb/crates/icydb-core/src/db/schema/live_schema_checkpoint.rs)
open this map; read-only `open_existing` alone does not initialize it.

The smallest credible change belongs in **IcyDB**: give this already-consolidated
control map a small separate governed allocation, leaving the commit buffer to
grow on demand. Do not split its record families into more maps. This removes
the fixed prefix coupling and can avoid roughly 16 MiB of early capacity when
the commit buffer is small. A smaller ICSS bucket cannot solve it.

This requires a new generated request and an explicit retained-layout migration
or fresh-image policy. Existing generated consumers and upgrade/recovery tests
must be qualified; no Storage public API change is needed. Moving the map must
not overwrite commit bytes or silently reset control records. Already-grown
stable memory cannot be shrunk by this change. Keeping the current layout is
simpler for existing deployed images, and a full commit buffer would erase most
of the saving. Confirm actual commit/control occupancy before prioritizing a
deployment migration. No sibling implementation or migration was attempted.

## 2. Requalify Canic's capacity reservation against its actual engine

**Verified:** [application receipt reservation](../../canic/crates/canic-core/src/storage/stable/intent.rs)
computes capacity using explicitly ICSS 0.7.2-specific constants: five entries
per node, a 2,378-byte chunk and 116 fixed bytes. The active map imports Storage,
and [receipt admission](../../canic/crates/canic-core/src/ops/storage/intent/mod.rs)
calls this reservation before admitting application receipts. Storage's current
framed B-tree and power-of-two block allocator have different geometry.

This is an active stale sizing assumption, **not a demonstrated capacity
failure or measured over-allocation**. The current
[capacity tests](../../canic/crates/canic-core/src/storage/stable/receipt_capacity_tests.rs)
already compare reserved and actual pages; they were inspected, not run here.

The smallest change belongs in **Canic**: retain the pre-admission reservation
guarantee, derive its bound for the selected Storage format and qualify it with
maximum-sized records, low occupancy, deletion/reinsertion and allocation reuse.
Replace the ICSS constants only when that bound is proven. No new shared package,
consumer API or retained layout is required. It could reduce excess reservation
or expose an under-reservation; neither outcome is established yet. Keeping an
unqualified old-engine bound invites drift as Storage evolves.

## Measure actual usage before consolidating more stores

Use existing [runtime diagnostics](../crates/ic-storage/src/runtime/mod.rs):
`memory_allocations()` supplies each key's pages; `diagnostics()` supplies data,
metadata, physical and unassigned pages, extents and live leases. Capture after
bootstrap, role initialization, representative work, churn and upgrade. Pair
these with collection lengths and payload bounds: reserved pages are not live
payload occupancy. This audit made no deployed-canister queries.

Preserve Canic's lazy initialization. Do not merge authority/lifecycle domains
just to reduce the count: saving one small collection's first 64 KiB page is
quite different from saving an ICSS 8 MiB bucket. Blob's fourteen declarations,
for example, do not imply 112 MiB of reserved buckets.

The existing [bucket experiment](../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md)
supports 1/8/128-page controls for **DFINITY ICSS**, not a Storage default switch.
Eight pages matched 128-page instructions across the selected integer trace,
but byte workloads had different tradeoffs. Storage's own 4 KiB payload block
rounding remains a separate profiling opportunity; changing its allocator format
is not justified merely by a memory-count audit.

## Reproduction and provenance

Evidence is retained locally in `target/platform-019-adoption/`: consumer search
hits, sibling HEAD/dirty-state inventory, incoming locks and patch, Platform
adoption checks and installed lifecycle logs. Main source snapshots: Canic
`0509ec870891` (399 changed paths), Blob `fd4649ebe5eb` (67), Relay
`db3746404f09` (20), IcyDB `b1d88ba9b939` (26), ic-crates `0851757145a7` (107).
Links above resolve into the sibling working trees and may change as their
pending work completes. Counts are source-derived; no sibling tests or release
qualification were run and no pending sibling work was modified.
