# Current handoff

The GitHub repository is now `dragginzgame/ic-storage` (same repository identity,
1427245576), and this checkout's origin uses the new URL. The local directory
is `ic-storage`. The adapters are now `ic-metrics-storage` and
`ic-logging-storage`, with matching Rust imports and package directories.
Release groups are `storage`, `metrics-storage` and `logging-storage`.
Published history remains at its original paths; only pending notes moved.
See [the naming adoption guide](../migration/storage-naming.md).

The maintainer's complete hard cut is implemented locally. Storage owns named
allocation, exclusive leases, backing/codec traits and all sibling-used collection
engines. The separate Memory package, fixed-slot manager, ledger reader and all
upstream stable-structures implementations/dependencies are removed. No legacy
reader or automatic reset remains. Hosts must adopt fresh/reset whole backing;
IcyDB must also replace old region framing admission with `ICMAP001`.

The supported roster is Cell, Map, Set, Vector and map regions. Log and Min-heap
were removed after a read-only sibling usage scan. Map/Set/regions share an owned
AVL tree with reusable blocks and staged node writes; Vector uses bounded slots.
The supported BTree collection names remain, but the implementation is AVL.
Public `Memory`, `Storable` and `Bound` identities now belong to Storage;
primitive numeric encodings are little-endian.

`StorageConfig`, `AllocationGrants` and opaque `AllocationId` values replace
manager geometry, numeric pools and slots. Namespace admission commits durable
names before issuing leases. Omitted names retain identity/data but require current
admission. Detached leases retain backing custody. Ordinary IO trusts custody;
construction, bootstrap, lease opening and nonzero growth validate the directory.
Generation advances for new names and extent growth, not an unchanged reopen.
See [allocation](../../crates/ic-storage/ALLOCATION.md),
[collections](../../crates/ic-storage/README.md) and
[the design](../design/dynamic-allocation.md).

The follow-up trim removes unused map/set convenience methods, the unused raw
Cell payload decoder and public layout constants with no sibling callers.
The bounded Cell envelope reader/header size remain for Auth/Scheduling consumers;
map first/last lookups remain for Canic. Lease tests use the production constructor
and typed growth; test-only construction switches and the second Memory adapter
are removed. Allocation-ID lookup uses the existing index instead of cloning
all diagnostic rows. Private directory candidates no longer take an encode/decode
round trip; final preparation still validates receiving budgets before mutation.

The locality follow-up skips unchanged ancestors during map replacement, retains
bounded pointer cursors for scans and derives logical extent indexes at directory
validation. Data IO spans contiguous physical extents instead of individual pages.
Public APIs and current owned record layouts are unchanged. Latest focused native
qualification passes 95 library tests (53 Storage, 29 Metrics, 13 Logging) and
strict library/test Clippy for all three packages. New tests bound replacement
writes and scan reads, compare mixed-direction ranges with the ordered model,
and verify extent-spanning IO after reopen. Evidence is in `target/storage-locality/`.
Earlier public-runtime/compile-fail proofs and the read-only sibling usage scan
remain in `target/storage-trim/`; they were not rerun in this follow-up.

Installed original-Wasm qualification covers all five collection types across
1,031 admitted names, allocation IO/bounds, metadata faults, poisoning, custody,
update/upgrade rollback and Logging checkpoint/text-budget/watermark restoration
on the prepared 16.1.0 server. Rebuilt original Wasm and the native runner pass
again for the locality follow-up; input hashes and logs are retained in
`target/storage-locality/runtime/`. This is lifecycle qualification, not a matched
instruction/cycle performance comparison; optimizer variants were not rerun.

The separate [map comparison](../../testing/runtime-qualification/crates/ic-storage-benchmarks/README.md)
now measures matched original Wasm against DFINITY 0.7.2, using the independent
lockfile and prepared 16.1.0 server. Both 256-row and 4,096-row matrices pass:
672 measured calls across both stacks, plus final full-map comparisons. Each
matrix covers 32-byte/1-KiB values, single/batched calls, uniform/hot keys and
three seeds. Evidence is retained in `target/storage-comparison/attempt.BNtMeL/`
and `target/storage-comparison/attempt.S6boOH/`, respectively. For single uniform
reads/replacements at 4,096 rows, Storage/DFINITY median instruction ratios are
1.62/1.96 (32-byte values) and 2.16/2.32 (1-KiB values). These are operation
intervals including each stack's allocator/governance, not complete endpoint
bills or isolated tree algorithms. Storage uses more instructions in these cases;
the comparison establishes a baseline, not a performance-equivalence claim.
Native workload tests, strict native/both-backend Wasm Clippy, the existing
runtime runner check, formatting, shell lint and production/dependency boundary
checks pass. The production lockfile is unchanged; no existing independent
dependency selection was updated. The benchmark is excluded from default
qualification and release gates.

The subsequent release-adapter integration includes this independent catalog and
lockfile in Storage's declared release metadata. It projects only the local
Storage version, with rollback under the existing transaction; adapter releases
preserve both files. This metadata-only follow-up does not change the measured
library or workload. All 78 native release-helper tests and strict helper Clippy
pass, including independent catalog/lock rollback and adapter-release preservation.
Release preparation restored the historical Persistence, Metrics Stable and
Logging Stable release documents at their original paths, resolving the missing
historical target previously reported by the expanded documentation scan.

The preceding hard-cut batch passed all package doctests, native/Wasm/runner
Clippy, graph/pin boundaries, 77 repo-tool tests and development archive consumers.
Its evidence remains in `target/owned-structures/` and
`target/package-qualification/attempt.pHaKKs/`. The subsequent naming adoption
passes 42 adapter tests, adapter doctests, 77 repo-tool tests, all three native
release-file plans, substitute release-adapter checks, formatting, script/workflow
lint, graph/pin boundaries and snapshot integrity. The independent native runner
also checks with the moved Logging protocol source. Evidence is retained in
`target/storage-rename/`. All three renamed development archives and their
native/Wasm consumers pass in `target/package-qualification/attempt.kpVxyd/`.
The full release gate was not rerun. Earlier prototype/consolidation proofs retain
their original source scope. Failed attempts remain retained.

The storage-safety follow-up adds typed codec/collection errors and fallible
iteration, a required cell load ceiling, checksummed cell/vector payloads,
reusable inactive directory arenas and contiguous-extent coalescing. Two-child
AVL deletion extracts its successor in one traversal. Formats are now
`ICROOTR1` / `ICDIRR01`, `ICCELL01` and `ICVECC01`; this extends the
pending fresh/reset-backing cut. Map framing remains `ICMAP001`.
Evidence in `target/storage-improvements/` covers 103 library tests (61 Storage,
29 Metrics, 13 Logging), ownership compile-fail/public-runtime tests, doctests,
strict native/Wasm Clippy and the owned benchmark caller. Installed original-Wasm
IO/upgrade/rollback, custody and Logging pass on 16.1.0 in
`target/storage-improvements/runtime/attempt.kLoeVb/`. Full snapshots still write
per mutation; no timing improvement, optimized-Wasm or full release-gate claim.

Known costs: directory mutations still clone/validate/write complete bounded
snapshots, now reusing inactive arenas. Application extents and abandoned tails
remain unreclaimed. Pointer cursors still reread complete
node records, including values for key scans; tiny scans may pay cursor setup
overhead. Derived directory indexes add one u64 per extent plus vector overhead.
Native writes are not transactions;
IC rollback is the installed atomicity boundary. No performance-equivalence or
full release-gate claim is made.

Pending notes are Storage 0.64.0, Metrics Storage 0.66.0 and Logging Storage 0.64.0;
manifest versions remain 0.63.0 / 0.65.0 / 0.63.0. The pending adapters include
Logging 0.24.0 and Metrics 0.26.0 supplier adoption plus the backing format cut.
No commit, release, publication or sibling checkout change was performed.
Downstream source adoption and publication are separate work.

Rust 1.99.1 remains the pinned compiler and floor. Reviewed Shared policy is
`e84252f5740bacd708359580455ea2b6fd67df38`; executable snapshot remains
`9275bf0de39ed941d81c47bab97eef7f6fc38df2` (VERSION 0.27.24, 136 payloads).
No vendored file changed. Platform 0.17.0 / Testkit 0.78.0 / server 16.1.0 remain
the qualified client/server pairing. Preserve earlier receipts, logs and archives.

The typed benchmark follow-up adds `u64`/`u64`, `u64`/`u128`, fixed 32-byte
keys/values, string keys/UTF-8 values and `u64`/Wire-record cases alongside the
original byte-vector cases. Four native workload tests, native benchmark Clippy,
both Wasm backend Clippy checks and focused formatting pass. The Storage dependency
still emitted warnings from the concurrent fallible-API migration; benchmark
calls now use its fallible signatures. No dependency or lockfile selection changed.
The expanded installed attempt `target/storage-comparison/attempt.iiwcwA/` was
refused before execution because four Storage files changed during compilation.
There are no qualified new timing results. Repeat `make benchmark-storage` after
that concurrent work settles. Focused logs are in `target/storage-types/`;
repository-wide formatting also encountered edits in progress outside the benchmark.
