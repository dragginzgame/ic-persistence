# Safety invariants

Storage owns allocation governance and collections. These invariants protect
allocation and collection integrity; they do not establish application schema
compatibility, controller authorization or data semantics.

## Allocation identity and admission

- A committed stable key retains its opaque allocation ID permanently. IDs and
  extents are never reassigned to another key. Omission does not reclaim storage.
- Only current namespace grants and successful host policy admission permit an
  allocation to open. Passive IDs and diagnostic rows grant no access.
- Directory validation checks unique names and IDs, disjoint nonempty extents,
  metadata separation, generation, checksums and configured resource bounds.
- Old manager, ledger and collection images are retired by an explicit hard cut.
  Invalid or foreign nonempty backing refuses; it is never silently initialized.

## Runtime and lease custody

One runtime claims each backing before reading or writing it. The claim survives
until the runtime and every detached lease are dropped. Each allocation permits
one live non-cloneable lease. Collections consume that lease; region views share
one retained lease and reload their tree root for each operation.

Hosts must provide truthful `MemoryBacking` identities and exclude raw aliases
and independent managers. Cooperating runtimes share the custody registry, but
Rust types cannot detect arbitrary raw accesses or a lying custom backing.
Construction, bootstrap, lease opening and nonzero growth validate the directory.
Ordinary IO trusts exclusive custody between those boundaries. Detected divergence
or a backing panic poisons all handles sharing the owner.

## Runtime IO bounds

The complete logical span is checked before backing IO. Reads and writes translate
only admitted data extents; directory metadata is never exposed as application
memory. Empty IO is permitted at the end of an allocation, but not beyond it.
Overflow and out-of-range spans fail before partial access. New data extents use
fresh zero-filled backing pages; abandoned native tails are skipped.

## Persistence and failure

Directory changes are prepared and encoded under budgets before growth or writes.
The snapshot is written before the protected root; cached authority is published
last. Rejected admission or ordinary capacity refusal does not publish a candidate.
New bindings and data growth advance generation; reopening unchanged bindings does
not. Inactive snapshot arenas are reused only after checking that they are
reserved metadata and do not overlap the active snapshot or application data.
There is no fallback generation or automatic repair.

Map/Set/region mutations stage affected B-tree nodes and external values and allocator metadata, reserve
capacity, write records, then publish changed root metadata. Vector writes publish length last.
Cell encodes and checks bounds before reserving capacity and replacing its value.
These are ordered writes, not native transactions. A native panic can leave partial
writes; poisoned owners must be dropped and retained state explicitly inspected.
IC message rollback provides atomicity for trapped updates and upgrades.

Record checksums detect accidental corruption; they are not authentication or
malicious-controller protection. Root loading is bounded. Map node validation is
lazy on accessed routing paths, with a traversal-depth cap; external values are
checked when read, so membership and key-only scans are not value-integrity audits; loading a root is not a full
audit of every node or free-list link. Caller codecs return typed encoding/decoding errors. A faulty implementation
can still panic; no panic interception substitutes for a fallible codec contract.
Cell/vector payload checksums run before decoding; cell loading also requires a
caller-selected byte ceiling.

## Resource limits and host policy

`StorageConfig` bounds allocations, extents, physical pages and Wire decoding.
Physical limits include reserved directory arenas and abandoned tails.
Collections enforce declared codec bounds and their own record limits. Hosts own
workload budgets, checkpoint cadence, retention and explicit fresh/reset installation.
Never clear governance alone while keeping application bytes to reinterpret as
managed storage. See [the allocation contract](crates/ic-storage/ALLOCATION.md)
and [collection limits](crates/ic-storage/README.md#current-storage-contract).
