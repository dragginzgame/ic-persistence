# IC Logging Stable

Optional bounded checkpoints for IC Logging, following Metrics Stable's explicit
load/save model. The package is unpublished. It requires the new validated
`LogSnapshot` contract prepared for Logging 0.17.2; registry-only qualification
must follow that upstream release.

IC Observability owns the engine, snapshot admission, append, retention, UTF-8
truncation and cursor scans. IC Persistence owns durable encoding and governed
Storage integration. Neither adapter depends on the other. Hosts retain clocks,
allocation declarations, stable identity, authorization, lifecycle and cadence.
No endpoint, recorder, timer or automatic checkpoint is installed.

## Host-owned storage

Use a map opened with an admitted `ic-memory::RuntimeMemory` handle. Choose
`StableBTreeMap::new(handle)` for known fresh storage, or `load(handle)` for known
retained storage. Never initialize an unknown/corrupt map as a recovery fallback.
`LoggingStore::new(map)` and `into_map()` transfer that same allocation lease;
drop every owner before reopening. Collection geometry, corruption/backing traps,
allocation and growth retain Storage/Memory's contracts.

```rust
use std::num::NonZeroUsize;
use ic_logging::{EntryInput, Level, Limits, Log};
use ic_logging_stable::{LogIdentity, LogKey, LoggingStore};
use ic_memory::{GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority,
    MemoryRequest, MemoryRuntime, SealedDeclarationSnapshot, VectorMemory};
use ic_storage::adapter::StableBTreeMap;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let pool = MemoryAllocationPool::new(vec![MemoryAuthority::new("app", "app.")?], vec![])?;
let declarations = SealedDeclarationSnapshot::new(vec![MemoryRequest::new("app", "app.logs.v1")?])?;
let mut runtime = MemoryRuntime::new(VectorMemory::default())?;
runtime.bootstrap(&declarations, &pool, &GenericAllocationPolicy)?;
let mut store = LoggingStore::new(StableBTreeMap::new(runtime.open_memory("app.logs.v1")?));
let key = LogKey::new("app", "worker")?;
let identity = LogIdentity { epoch: 0 };
let limits = Limits { max_entries: NonZeroUsize::new(128).unwrap(),
    max_message_bytes: 192, max_label_bytes: 32 };
let mut log = store.load(&key, identity, limits)?.unwrap_or_else(|| Log::new(limits));
log.append(EntryInput { created_at_secs: 10, level: Level::Info,
    source: "worker", topic: Some("delivery"), message: "intent recorded" })
    .map_err(|error| format!("append refused: {error:?}"))?;
store.save(&key, identity, &log)?;
drop(store);
let reopened = LoggingStore::new(StableBTreeMap::load(runtime.open_memory("app.logs.v1")?));
let restored = reopened.load(&key, identity, limits)?.unwrap();
assert_eq!(restored.snapshot(), log.snapshot());
# Ok(())
# }
```

Only `None` is absence. Malformed, wrong-generation or wrong-configuration rows
are typed refusals. A stored empty history still retains both append watermarks;
it never becomes absence. Sequence exhaustion remains exhausted after reopen.
The host supplies the same epoch across ordinary upgrades and owns generation
identity; schema validity cannot establish authenticity or instance continuity.

## Bounds and durable V1 contract

Namespaces/names contain 1..=64 / 1..=128 ASCII bytes from letters, digits,
`._/-:`. Each key encodes namespace, NUL, name. The host bounds key cardinality
and keeps authority outside these labels. Each log admits at most 1,024 configured
entries, 4,096 message bytes and 128 bytes per source/topic label. The complete
encoded record is at most 65,536 bytes; not every combination of maximum settings
fits that byte ceiling. Oversized saves fail before map mutation. Snapshot copies
and checkpoint comparisons scale with retained history, unlike cursor scans.

V1 uses `ICL` plus byte `1`, an eight-byte epoch, three four-byte fixed limits,
optional next sequence and last timestamp, then a four-byte row count. Optional
values use a 0/1 byte followed by eight bytes when present. All integers are
little-endian. Each oldest-first row carries sequence/time (eight bytes each),
severity (0..=4 for Debug/Info/Ok/Warn/Error), a 0/1 truncation flag, source text,
a 0/1 topic presence flag and optional topic text, then message text. Text uses
a four-byte UTF-8 byte length. Rust layouts and enum discriminants are not stored.
Decoding checks bytes, counts, lengths, flags, UTF-8 and full consumption before
canonical `Log::restore` admission. Unknown formats never select another reader.

Save verifies the old row and rejects decreasing/exhaustion-clearing watermarks,
rewritten issued rows and resurrection of pruned rows. An unchanged next sequence
cannot invent a new timestamp watermark. Identical checkpoints skip stable writes.
Replacing the same row reuses Storage's collection allocation; pruning never
accumulates a second archival row and never removes the watermark checkpoint.

The host must load current state before changes and establish its own identity.
These consistency checks do not authenticate a checkpoint or detect every reset
followed by enough new appends. Explicit `remove` admits the old identity/limits;
for a deliberate reset, save with a new epoch in the same update without an await.
Corrupt records require explicit host map administration; there is no silent reset.

Checkpoint in successful update execution. Await boundaries, IC trap rollback,
query non-persistence and whole-application upgrade acceptance require installed
consumer qualification. Native VectorMemory tests prove codec/storage behaviour,
not IC effects or instruction/cycle costs. Diagnostic history never authorizes
retrying uncertain paid effects or proves that a remote effect completed.
