# IC Metrics Stable

This package is maintained and released by IC Persistence alongside Memory and
Storage. The dependency-free `ic-metrics` arithmetic crate remains independently
maintained in [IC Metrics](https://github.com/dragginzgame/ic-metrics).
The [migration guide](https://github.com/dragginzgame/ic-persistence/blob/main/docs/migration/ic-metrics-stable.md) records source
provenance, version selection and unchanged API/storage contracts.

Optional durable counter, aggregate, summary and histogram snapshots in **one stable
B-tree map per canister**. Multiple components share that map with permanent
namespace/name keys. Adding this package is the opt-in: `ic-metrics` alone stays
allocation-free, dependency-free, `no_std` and volatile. This adapter uses `std` and IC Storage’s governed upstream-map facade,
allocates bounded records and supports native hosts and Wasm with Rust 1.91.

The host owns one `ic-memory` allocation pool and bootstrap. It declares one
permanent metrics-store key, such as `app.metrics.v1`, and admits that allocation
before opening the map. Follow the
[IC Memory 0.50 allocation and bootstrap guide](https://github.com/dragginzgame/ic-persistence/blob/f452b5cf9b26a25ee2fb69f4941c079e9c85f4e7/crates/ic-memory/README.md#quick-start-for-developers)
for the selected dependency's declaration and lifecycle APIs.

Memory and Storage are published from IC Persistence with their existing crate
names. Hosts select `ic-metrics-stable`, `ic-memory` and `ic-storage` from the same
Persistence release line when supplying a map to this adapter. The adapter now
inherits Persistence's version; `ic-metrics` retains its independent 0.16 line.
Hosts passing arithmetic values to the adapter must also select
`ic-metrics = "0.16"`; 0.15 values have a different Rust type identity.
Earlier dependency generations have
different Rust handle/map type identities; update them together. The 0.50
alignment preserves the preceding library implementations and stored formats.

After bootstrap, the host opens `ic_storage::adapter::StableBTreeMap` with one
admitted memory handle and passes that public collection to `MetricsStore::new`.
Components share access to that store;
namespace/name keys distinguish rows without allocating an ID per metric or
component. `ic-memory`'s governance allocations are separate. This package never
allocates memory IDs, registers keys, creates a global store or installs lifecycle
hooks.

Metrics' V1 record layout is separate from Memory's allocation-ledger format.
Memory 0.50 retains the allocation format introduced in 0.48 and preserved by
0.49's `serde_cbor` backend replacement. This alignment requires no data reset.
Earlier ICMSNAP1/ICMREC01 images remain retired and require host-owned fresh/reset
backing under Memory's version-bound disposition. Metrics neither converts those
images nor clears memory on failure.
An unchanged Metrics record or B-tree format does not establish allocation-ledger
compatibility or authorize clearing existing memory.

## Store operations

Use the concrete operations for each metric family. Loads need only the row key,
identity and, for histograms, finite bounds; they do not take a dummy observation.

| Arithmetic type | Load | Checkpoint |
| --- | --- | --- |
| `MeasurementCounter` | `load_counter(key, identity)` | `save_counter(key, identity, value)` |
| `MeasurementAggregate` | `load_aggregate(key, identity)` | `save_aggregate(key, identity, value)` |
| `MeasurementSummary` | `load_summary(key, identity)` | `save_summary(key, identity, value)` |
| `MeasurementHistogram<N>` | `load_histogram(key, identity, bounds)` | `save_histogram(key, identity, value)` |

Histogram loads admit at most 256 strictly increasing bounds before reading a row;
invalid expected bounds return `StoreError::InvalidState(InvalidBounds)`.
Serialization, kind selection and continuity dispatch are private implementation.

`MetricsMap<M>` names the exact `ic_storage::adapter::StableBTreeMap<Vec<u8>,
Vec<u8>, M>` type. Hosts may use either spelling; this is a type alias, with no
collection wrapper or independent initialization policy. `MetricsStore::new(map)`
and `store.into_map()` transfer that same collection and its allocation lease.
Hosts select `StableBTreeMap::new(handle)` for known fresh storage or
`StableBTreeMap::load(handle)` for a known retained map. The store constructor
wraps the supplied collection without touching storage. The generic parameter
remains the runtime's backing memory, and the B-tree format and Metrics V1 rows
are unchanged. No custom tree format or additional memory ID is introduced.

`into_map` transfers the collection and its allocation lease to the host for
administration. Drop the store or returned map before reopening the allocation.
Direct map changes are host-owned and must preserve valid Metrics rows; subsequent
store operations still validate stored records at their ordinary boundaries.

This complete native example owns bootstrap once, opens only admitted handles,
checkpoints all four families and reopens the same map. It proves storage/codec
behavior, not IC upgrade execution or instruction/cycle costs:

```rust
use ic_memory::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryRequest,
    MemoryRuntime, RuntimeOpenError, SealedDeclarationSnapshot, VectorMemory,
};
use ic_metrics::{MeasurementAggregate, MeasurementCounter, MeasurementHistogram, MeasurementSummary};
use ic_metrics_stable::{MetricIdentity, MetricKey, MetricsStore};
use ic_storage::adapter::StableBTreeMap;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
const STORE_KEY: &str = "app.metrics.v1";
let pool = MemoryAllocationPool::new(
    vec![MemoryAuthority::new("app", "app.")?], vec![],
)?;
let declarations = SealedDeclarationSnapshot::new(vec![MemoryRequest::new("app", STORE_KEY)?])?;
let mut runtime = MemoryRuntime::new(VectorMemory::default())?;
runtime.bootstrap(&declarations, &pool, &GenericAllocationPolicy)?;
let mut store = MetricsStore::new(StableBTreeMap::new(runtime.open_memory(STORE_KEY)?));
assert!(matches!(runtime.open_memory(STORE_KEY), Err(RuntimeOpenError::AlreadyOpen { .. })));
let identity = MetricIdentity::new("instructions", 0)?;
let key = MetricKey::new("canic", "endpoint.create")?;
let mut summary = store.load_summary(&key, &identity)?.unwrap_or(MeasurementSummary::EMPTY);
summary.record(42);
store.save_summary(&key, &identity, &summary)?;

let bytes_key = MetricKey::new("app", "processed.bytes")?;
let bytes_identity = MetricIdentity::new("bytes", 0)?;
let mut bytes = store.load_counter(&bytes_key, &bytes_identity)?.unwrap_or(MeasurementCounter::ZERO);
bytes.add(32_768);
store.save_counter(&bytes_key, &bytes_identity, &bytes)?;
let aggregate_key = MetricKey::new("app", "batch.size")?;
let items = MetricIdentity::new("items", 0)?;
let mut aggregate = store.load_aggregate(&aggregate_key, &items)?.unwrap_or(MeasurementAggregate::EMPTY);
aggregate.record(3);
store.save_aggregate(&aggregate_key, &items, &aggregate)?;
let histogram_key = MetricKey::new("app", "batch.distribution")?;
let bounds = [0, 10];
let mut histogram = store.load_histogram(&histogram_key, &items, &bounds)?
    .unwrap_or(MeasurementHistogram::new(bounds)?);
histogram.record(3);
store.save_histogram(&histogram_key, &items, &histogram)?;
let map = store.into_map();
assert!(matches!(runtime.open_memory(STORE_KEY), Err(RuntimeOpenError::AlreadyOpen { .. })));
drop(map);

let reopened = MetricsStore::new(StableBTreeMap::load(runtime.open_memory(STORE_KEY)?));
assert_eq!(reopened.load_summary(&key, &identity)?, Some(summary));
assert_eq!(reopened.load_counter(&bytes_key, &bytes_identity)?, Some(bytes));
assert_eq!(reopened.load_aggregate(&aggregate_key, &items)?, Some(aggregate));
assert_eq!(reopened.load_histogram(&histogram_key, &items, &bounds)?, Some(histogram));
# Ok(())
# }
```

Production hosts retain one bootstrapped runtime and open the public IC Storage
collection for the permanent metrics key. IC Storage requires
`ic_memory::RuntimeMemory` when opening its map; raw backing cannot bypass
allocation admission. Runtime handles cannot be cloned, and another live open
through the same runtime returns `RuntimeOpenError::AlreadyOpen`. Hosts still
prevent aliased backing access across runtimes. The store and a map returned by
`into_map` retain that lease until dropped.

Hosts own the new-versus-retained format decision. Never use IC Storage's fresh
constructor as an unknown/corrupt-store recovery fallback; its retained `load`
constructor does not fall back to initialization. Allocation/collection corruption
and capacity traps remain upstream contracts; typed adapter errors cover
identifiers, values and numeric admission.

## Durable contract

- Namespace/name keys are stable ASCII identifiers, up to 64/128 bytes. Units
  use the same alphabet and have a 64-byte limit. These namespaces separate rows;
  the canister still owns authorization and label cardinality.
- Every V1 record has a format discriminator, kind, unit, reset epoch and explicit
  little-endian state. Kinds 1/2/3 remain aggregate/summary/histogram; kind 4 is
  a scalar counter's 16-byte `u128` total followed by one overflow byte (0 or 1).
  Counters have no sample count, maximum or latest value. Existing kinds retain
  their exact layouts; this adapter introduces no record-format reset. An older
  adapter cannot read kind 4, so hosts must retain counter-aware code when reopening
  those rows.
  Records retain `u128` totals, maxima/latest, count/total overflow
  flags, histogram bounds, disjoint bucket counts and their flags. It never copies
  raw Rust struct memory. Histograms admit at most 256 finite bounds; records are
  limited to 4,608 bytes at the codec boundary. Constructor-admitted units and
  shape-admitted encoders are bounded together at compile time; the largest
  current encoded record is 4,474 bytes. Stored bytes still undergo independent
  size, envelope and numeric validation.
- Load checks kind, unit, epoch and exact histogram bounds, then restores through
  the core's typed numeric admission. Unknown formats, truncation, trailing bytes
  and malformed state are errors; they never silently create an empty metric.
- Save replaces a full snapshot rather than adding it again. Repeated identical
  saves validate the retained row and skip encoding/map writes when unchanged.
  Numeric regressions and clearing overflow flags are rejected before writing.
  With exact sample counts, the added total must fit the added observations'
  maximum, the summary's latest value and the histogram's added bucket ranges.
  A new maximum must be attainable among those additions. Impossible transitions
  return `StoreError::StateRegression` without writes. Overflowed sample counts
  retain the conservative monotonic checks because exact additions are unknown.
  Supply the latest retained state before recording; numeric checks do not
  establish sample attribution, ordering or provenance.
- Deliberate reset/retirement uses `remove` with the old identity. It verifies the
  envelope and discards the payload explicitly, including unreadable payloads.
  For a reset, save the replacement with a new epoch in the same update message,
  without an await between operations. It never retires/reuses the allocation ID.

Batch observations within an update message and checkpoint at an attribution
boundary chosen by the host. Stable writes during successful update execution
survive upgrades; query writes do not persist, and writes in a trapping message
roll back with that message. Async callbacks are separate message boundaries.
Checkpointing less frequently explicitly permits loss since the last checkpoint.
Neither persistence nor an upgrade establishes counter/call-context continuity;
unfinished spans must be re-established by the consumer.

Host-owned endpoints, resets, sample admission and synchronization remain local.

For diagnostic byte or event totals, use `load_counter`, start an absent row
with `MeasurementCounter::ZERO`, add only admitted exact quantities, then checkpoint at the host's
update-message boundary. Absent and stored zero remain distinct. Exact
`u128::MAX` and actual overflow remain distinct too; an overflowed counter cannot
later clear its flag or decrease without explicit removal and a new reset epoch.
These diagnostics never replace authoritative balances, quotas or ledger debt.

The native example verifies codec/storage behavior, not IC checkpoint costs or
upgrade execution. Hosts choose which completed work contributes and when to
write; persistence does not establish attribution or reset continuity.
Stable map reads/writes add allocation and IC costs. The frozen
[Memory 0.49 comparison](https://github.com/dragginzgame/ic-metrics/blob/main/docs/evidence/memory-0147.md)
checks retained upgrades from Memory 0.48, query discard, trap rollback, leases
and reset, with matched IC checkpoint and bootstrap measurements.
Batch observations before checkpointing at the
owning message boundary; measured costs depend on workload, map state and cadence.
The prior [IC store receipt](https://github.com/dragginzgame/ic-metrics/blob/main/docs/evidence/stable-current-0146.md)
retains its Memory 0.48.2 / Storage 0.5.6 identities. Neither fixture qualifies a
later graph or an application's workload, lifecycle and deployment decisions.
The arithmetic-only cost evidence and native reopen tests retain their separate
scope and do not qualify live consumer execution.

The record byte budget limits codec admission after IC Storage reads a row.
It is not a bound on allocation while reading a host-supplied oversized value.
Hosts administering the public byte map must preserve its Metrics-only row
contract; underlying collection and allocation failures retain Storage semantics.
