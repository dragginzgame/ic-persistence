# Copied ICSS collection baseline — 2026-11-05

The maintainer selected a hard cut of all collection engines to copied published
ICSS 0.7.2 source, then explicitly chose a source copy instead of a production
dependency. The two collection layers are private `src/engine` and public
`src/adapter`; they do not retain two algorithms for the same collection.
Storage's named allocation, policy, custody, lease and codec ownership remain.

## Implementation and compatibility

[Provenance](../../storage/upstreams.md) and [original hashes](../../storage/icss-source.json)
identify the copied subset and deliberate adaptations. There is no copied
MemoryManager, independent ID allocator or public raw-memory constructor.
Map and Set share the B-tree; region views use that engine over bounded governed
memory. Vector uses fixed-stride slots with scalar iteration; Cell caches a value.
The old independent tree/block allocator, record wrappers, node cache and Vector
windows/epochs are deleted; the historical study retains compact conclusions. Iterator
facades forward specialized skip/count methods to the engine rather than placing
a general Map adaptor in front of them.

All earlier Storage collection images require explicit host rebuild/reset or
fresh allocation. No old Storage decoder, old ICSS tree reader, migration or
implicit initialization remains. Current engine framing is BTR2 with explicit
page-size metadata, SVC1 and SCL1. Directory bytes/checksums are unchanged.
Successful-call API shapes and application codecs remain; engine and codec
failures now trap. There are no collection checksums. A positive node-cache
ceiling remains a harmless hint, not a cache enabler; benchmark runs reject it.

Six native same-image tests exchange Map (splits/overflow/removal), Set, fixed
and variable Vector, Cell and governed region images with independently published
ICSS. Tests write through one implementation, reopen/update with the other, then
reopen through Storage. They use matching Storage application codecs and bounds.
Stock unsigned ICSS defaults use big-endian bytes; Storage retains little-endian.
Engine-format interoperability does not promise arbitrary default-codec exchange.

## Safety adaptations

Cell retains the caller's pre-allocation byte ceiling. Variable Vector lengths
are admitted against the bound before payload allocation. A persisted allocator
flag uses validated u8 rather than arbitrary stored bytes materializing a Rust
bool; valid persisted bytes are unchanged. Native mutation unwinds poison the
shared runtime owner, including panics after partial writes or denied engine
growth. The guard is absent on Wasm: IC execution rollback supplies atomicity.
Do not recover by catching a native mutation panic and reusing cached views.

The current engines intentionally retain ICSS's traversal/allocation approach.
This port is not full-image validation or a guarantee of bounded work under
arbitrary corrupt links. Host framing/schema admission and native backing recovery
remain explicit obligations. [Safety contract](../../../SAFETY.md).

## Validation

Evidence is retained under `target/icss-baseline-reset/`:

- `core-tests.log`: 60 core library tests, including collection models, generated
  histories, bounds, governance, custody and failure checks.
- `adapter-tests.log`: 13 Logging Storage and 29 Metrics Storage tests.
- `custody-compile-fail.log`: both lease/custody compile-fail fixtures.
- `reference-and-driver-tests.log`: same-image reference tests and benchmark model/
  driver tests; the six reference checks also appear in `interoperability-3.log`.
- `collection-tests-iterator-fix.log`: 23 selected collection tests after the
  forwarding fix, including one decode and one payload read for direct skips,
  plus zero payload reads/decodes for count. The preceding 60-test core pass
  includes the other 38 unchanged runtime/codec tests.
- Strict native core, benchmark/reference tests and root-graph Wasm core Clippy.
- `installed-runtime-final.log`: original-Wasm Storage-only installed proof on prepared
  Testkit 0.79.0 / PocketIC 16.1.0: collection IO, read-only Cell admission,
  1,031 named allocations, successful upgrades and trapped decoder/update/upgrade
  rollback, including fixed and variable Vector payload replacements.

Rust/Cargo 1.99.1 are selected. The independent lock remains byte-identical to
this reset's saved incoming snapshot, preserving its earlier serde_bytes edge.
The root lock acquired additional supplier selections after the initial snapshot;
these are preserved, not reverted: Wire 0.17.2, Runtime 0.19.2 and Build/Host
0.74.4. Root native/Wasm proofs use that graph. The installed fixture and matched
comparison retain their independent Wire 0.17.0 / Runtime 0.19.0 graph. No locked
validation command changes dependency selection. No release/version,
commit, push, sibling migration or backing reset is performed. Full release gates,
optimized-Wasm variants and other host qualification are not claimed.

## Matched performance baseline

The default matrix passed 3,196 calls across 30 cases / 54 type-size combinations,
256 rows (Cell one), uniform access, batches 1/32 and one seed. A general `.map()`
facade defeated ICSS's specialized nth/nth_back skip; forwarding fixes passed
824 affected Vector/Set reruns. The 480-call Auth preset also passed. Its full
outer-decode controls did not reproduce Canic's reported population-dependent
cost jump. Canic host preparation and Auth's inner session decoder were not measured.

Original local runs are `target/storage-comparison/attempt.difIFu/`, `UZTbp1/`
and `Q4kJp3/`; complete tables and repeated identity exports were retired from
documentation in the authorized evidence cleanup. Target artifacts remain
transient, not a durable archive. The selected absolute results below remain;
hashes/summaries do not replace missing raw evidence. The broader
[type matrix](benchmark-full-catalogue-2026-11-05.md) has its own source and scope.

Selected instruction counts follow; controls are not subtracted:

| Case / operation | Storage | ICSS 0.7.2 |
| --- | ---: | ---: |
| u64 Map Get, single | 99,459 | 93,957 |
| u64 Map Replace, 32 | 1,192,745 | 913,771 |
| u64 Vector Get, single | 81,018 | 81,144 |
| 4 KiB Vector Get, single | 169,629 | 165,274 |
| Wire-record Cell cached Get, single | 488 | 620 |
| 32-byte Region Get, single | 109,935 | 94,569 |
| Auth first-record 2 KiB Get, 2,048 rows, update | 88,435 | 86,094 |
| Same Auth case with untimed pre-read | 8,435 | 6,094 |

These measure configured stacks, including governance, adapters, codec choices
and backing allocation; the copied algorithms alone do not account for every
difference. Map ratios span 1.0091–1.4649, Set 1.0332–1.4636, Vector
0.9078–1.4477, Cell 0.7773–1.2125 and Region 1.0434–2.0098 over non-control
paired cells. Ratios are individual cells, not an aggregate workload or fleet score.
The ICSS control explicitly uses eight-page buckets, not its 128-page default.
No universal improvement or 2× whole-call speedup is established.

One retained baseline cost is Vector `iter().last()` scanning the population:
a 31-byte case costs about 20.8M instructions in both implementations. This is
now a shared baseline behavior, not justification to silently restore the old
shortcut during this reset. The native forwarding regression separately protects
specialized direct skips and constant-work count.

Larger populations, hot/shuffled histories, 1 MiB payloads and other ICSS bucket
sizes remain separate qualifications; their historical runs are not reused as
proof of current performance. Canic configuration and Auth's inner session codec
are outside scope. Source alignment and first-access dispatch can still move
80k/160k charges between reads or executions; the historical
[diagnostic](vector-stable-read-charges-2026-11-05.md) remains relevant. The next
optimization pass can isolate adapter/backing overhead against this baseline.
