# Dynamic allocation directory

The maintainer selected named allocations with automatically assigned opaque
identities, replacing the fixed manager-slot domain. Storage owns both the
allocator and governed collections. The production runtime now uses the owned directory and sparse leases. The
fixed-slot manager and separate ledger reader are retired; namespace admission
belongs to the runtime and grants no access before durable binding commitment.

## Implementation

The [directory](../../crates/ic-storage/src/runtime/directory_model.rs),
[admission](../../crates/ic-storage/src/runtime/admission.rs) and
[runtime](../../crates/ic-storage/src/runtime/mod.rs) are production modules.
Focused directory tests run with:

```sh
cargo test -p ic-storage --locked --offline --lib runtime::directory_model
```

The directory maps checked stable keys to opaque u64 allocation identities and
ordered physical page extents. Interleaved growth does not require contiguous
physical allocation. IDs are monotonically assigned, never reused, and
independent of key ordering on later opens. Omitted keys retain ownership.
There is no fixed 256-entry table.

StorageConfig admits explicit allocation-count, extent-count, physical-page and
Wire resource budgets. Default limits are 4,096 allocations, 16,384
extents, 1,048,576 pages, 4 MiB of encoded directory bytes and 16 MiB of Wire
allocation. Physical-page accounting includes metadata and abandoned tails.
Hosts may select different explicit limits; these ceilings are not reserved
space. The u64 ID width does not promise billions of supported allocations.

Wire owns logical serialization and resource admission. Structural recovery
validates key grammar, canonical unique IDs, unique names, identity watermarks,
physical bounds, arithmetic and pairwise extent disjointness. One exclusive store
borrow covers backing verification, candidate preparation and publication; no
candidate is transferred between owners or retained across mutations. Preparation
failure leaves the base unchanged. The owner's budgets are enforced before
physical growth or writes.

## Durable commit

[Persistence](../../crates/ic-storage/src/runtime/directory_model/persistence.rs)
reserves the first physical page for a fixed 64-byte `ICROOTR1` root. Each commit
selects an inactive arena for an `ICDIRR01` Wire snapshot. If none fits, it
reserves a geometrically sized arena beyond existing data and metadata, admitting
the snapshot including its own reservation. All arenas are recorded and validated
as disjoint from application data; the active arena is never overwritten. The root binds generation,
record offset, exact byte length and physical end. Storage's FNV checksum covers
the root and record separately for accidental corruption detection; it is not
an authenticity mechanism against hostile backing edits.

All budgets and offsets are checked before backing growth. Growth refusal
preserves the root, bindings and cached extents. The snapshot is written first,
then the root; only after both writes return does the owner publish new state.
The owner is poisoned before backing IO. An interrupted write cannot expose its
tentative state through that owner. Reopening reads and validates the root and
snapshot without writes, fallback guessing or automatic reset. A partial native
root write can make reopening fail; native writes are not transactional. A full
root write followed by a panic may be recovered by a new owner.

Failed native writes can leave physical pages beyond the committed watermark.
New allocations skip that abandoned tail rather than exposing old metadata as
zeroed application pages. Committed inactive snapshot arenas can be reused,
but abandoned pages and application extents are not reclaimed. Create accepts
only genuinely empty backing.

## Sparse leases and data IO

[Leases](../../crates/ic-storage/src/runtime/directory_model/leases.rs) use a
sparse set of live u64 IDs. One lease per allocation is allowed, and only a
committed name can be opened. The runtime uses the backing-claim
registry, preventing two owners from claiming the same backing. A shared owner
retains custody through the final detached lease; claims are released before
the allocation identifying the backing is destroyed.

Reads and writes validate the complete logical range, then split it across
physical page extents. Caller bounds errors do not perform partial IO. Backing
panics poison all shared leases. Ordinary data IO trusts exclusive custody;
full directory checks occur at lease opening and allocation commits. This
preserves the maintainer's selected checked-boundary contract. Growth commits
its new extents durably before increasing a lease's visible size.

## Costs and qualification

The directory clones one candidate image and writes a complete snapshot per mutation.
Metadata reservation encodes that image under Wire limits until the selected arena
fits, then validates it and constructs its lookup indexes once. Backing verification
still independently validates the retained snapshot at the mutation boundary.
Inactive arenas are reused; a stable snapshot size needs two arenas. Superseded
smaller arenas remain reserved and may be reused if a later snapshot fits.
Contiguous growth coalesces allocation extents, limiting snapshot growth.
All metadata and abandoned tails count in the physical budget; application
allocation reclamation remains future work. Ordinary data IO does not reread the directory.
Map, Set and regions share the owned reusable-block B-tree engine; Vector uses
bounded indexed slots. These engines use Storage's own Memory/Storable traits.
There is no upstream stable-structures dependency or legacy-format fallback.

Native tests cover recovery, IO, custody and failure boundaries, including 1,024
live leases and multi-page snapshots. Public runtime tests qualify namespace
admission and retained-name ownership. Installed qualification must use newly
built fixtures; earlier manager receipts do not qualify this hard cut. See the
[current handoff](../status/current.md) for actual completed checks and evidence.
