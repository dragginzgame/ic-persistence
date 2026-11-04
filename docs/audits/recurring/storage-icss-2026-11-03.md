# Storage versus ICSS: re-audit and bounded follow-through

## Scope and identity

Requested re-audit and implementation, 2026-11-03. Source base `7c5cf83660cc`
plus the pending Shared Tooling adoption, dependency selections and memory study;
those edits were preserved. Compared published **ICSS 0.7.2** and the separately
pinned DFINITY **b5d1650463f477affb3d358e86b1113953fc9937** research source.
Neither is an older version of Storage. Production has no ICSS dependency.

Applied the [shared audit contract](../../../audits/README.md) and selected
[code-hygiene questions](../../../audits/code-hygiene.md) to collection decoding,
allocator lifecycle, directory publication and benchmark validity. Shared revision
is `04a87b366eae3e63b9f032c0b891c17bdc59e69b`. The local hygiene overlay SHA-256 is
`1811b6058decfff2a8de4bba5b5fee71c2b3c08e1076935bc3a2616b57f0e8e2`;
AGENTS.md is `e825748ce6196193f352b0e5bdba0f2a2d4674d6604d722282f1f97ca216f2f9`.
Current product authorities are [SAFETY](../../../SAFETY.md) and the
[named-directory contract](../../current-ledger.md); obsolete module paths in
the historical overlay are not current implementation authorities.

Verdict: **PASS WITH FINDINGS** within these inspected boundaries. This is not a
whole-product safety certification. Release adapters, sibling migrations, all
corruption inputs, macOS execution and installed upgrade/rollback qualification
are outside this pass. Performance conclusions distinguish retained measurements,
fresh native experiments and installed IC executions.

## Differences worth preserving

| Boundary | Storage | ICSS controls |
| --- | --- | --- |
| Ownership | Named allocations, namespace grants, exclusive checked leases and backing custody | Generic Memory and caller-owned MemoryManager IDs; correct separation is the caller's obligation |
| B-tree updates | Degree six; compact routing, separately checked external values, staged writes | Degree six; node values and overflow pages have different read/write amplification |
| Integrity | Fallible codecs, length/bounds checks and noncryptographic record checksums | Different validation and panic contracts; Cell/Vector payload integrity is not equivalent |
| Cache | Disabled by default; optional retained-byte budget | Published 0.7.2 has no node cache; pinned b5d1650 defaults to 16 slots |
| Lifecycle | Explicit empty create, read-only load, explicit reset; current format hard cut | Established ICSS formats and compatibility history; initialization behavior differs |
| Structures | Cell, Map, Set, Vector, map regions | Also Log and MinHeap; no verified local demand justifies adding them for parity |

Neither collection engine requires Candid encoding. Storage uses IC Wire for
its owned directory/record metadata and `Storable` for application payloads.
The comparisons use matched concrete codecs. Cache-disabled comparison results
do not describe the default cached behavior of DFINITY b5d1650. Retained large
map measurements favor Storage's selective writes, while several small-value
and bulk Cell/Vector cases favor ICSS. Neither is universally faster or safer;
the contracts and measured workloads differ. IC rollback supplies trap atomicity;
Storage's staged native writes are not a second transaction system.

## Findings and decisions

### MEDIUM: bulk growth coverage did not exercise larger values

The benchmark's `payload_for` deliberately capped growth at 64 KiB, then kept
larger initial values unchanged. The README disclosed this; it was a coverage
limitation, not an engine corruption bug. The earlier audit overstated it as an
undocumented bug. Both canisters and the host model shared that policy, so result
agreement could not detect the missing growth scenario.

The corrected [benchmark](../../../testing/runtime-qualification/crates/ic-storage-benchmarks/src/lib.rs)
grows 64 KiB to 256 KiB and 1 MiB to 1.5 MiB. Tests require strict growth and
admit the encoded response under both Wire budgets and the IC reply ceiling.
An initial 2 MiB target passed expanded native budgets but failed installed
Shrink, which returns the old value: framing exceeded the IC's 2 MiB reply
limit. That failed evidence is retained; the final target preserves existing
Wire limits. Vector's 1 MiB bounded fixture does not select Grow.

Earlier bulk Grow rows and downstream allocator state are not comparable with
the corrected workload as a speedup. Historical evidence remains unchanged.

The corrected installed bulk run passed all **172 measured calls**, including
results, lengths and final contents. One sample, one round, disabled map caches,
ICSS 0.7.2 with eight-page buckets, PocketIC 16.1.0; this is not a statistical
throughput study. [Complete CSV](../../measurements/storage-icss-bulk-2026-11-03.csv).
For 1 MiB → 1.5 MiB Grow, Map costs 37,720,652 Storage instructions versus
76,026,494 ICSS; Cell costs 18,585,336 versus 5,739,124. These compare different
current implementations and integrity contracts, not gains caused by this patch.

### LOW: borrowed map-entry decoding unnecessarily copied encoded bytes

[MapEntry::value](../../../crates/ic-storage/src/adapter/tree.rs) cloned the whole
buffer before calling the codec. ICSS's
[LazyEntry::value](https://github.com/dfinity/stable-structures/blob/b5d1650463f477affb3d358e86b1113953fc9937/src/btreemap/iter.rs)
passes borrowed bytes. Storage now also checks bounds and passes `Cow::Borrowed`.
Consuming entry methods still transfer the owned buffer. Regression coverage
checks repeated borrowed decoding, consuming decoding and codec errors for inline
and external values.

No public signature, stored bytes or entry lifetime changes. This removes a
Storage-side allocation/copy for codecs able to decode borrowed input; a codec
returning an owned Vec still needs its own copy. Existing scan benchmarks consume
`into_pair`, so their earlier slowdown is not evidence for this fix's speedup.
Truly lazy external IO would require a different entry lifetime contract and is
not adopted without a demonstrated consumer need.

### MEDIUM: variable-size allocation history was underrepresented

[Tree allocation](../../../crates/ic-storage/src/adapter/tree.rs) retains a
32-byte inline threshold, minimum 256-byte external blocks, a 20-byte record
prefix, power-of-two classes and splitting without coalescing. ICSS uses
[fixed chunks plus node overflow](https://github.com/dfinity/stable-structures/blob/v0.7.2/src/btreemap/allocator.rs).
The retained study verifies abrupt capacity changes at 33 bytes and immediately
above payload-plus-header power-of-two boundaries. Same-size churn plateauing
cannot establish behavior under changing sizes.

Extended the existing generated histories across 236/237 and 4076/4077 bytes.
A variable-key history now combines variable routing sizes, 31/32/33-byte values,
external boundaries through 65516/65517, deletion, cold reopening, three cache
budgets and independent live/free-range checks. The shared-backing experiment
adds six matched size-changing rounds after its unchanged same-size phases.

Fresh native results, 64 allocations at the end of the sixth replacement round:

| Engine | Physical pages / MiB | Bytes written in that round |
| --- | ---: | ---: |
| Storage | 3,813 / 238.31 | 97,577,577 |
| ICSS 0.7.2, 1-page buckets | 2,291 / 143.19 | 966,683,129 |
| ICSS 0.7.2, 8-page buckets | 2,689 / 168.06 | 966,683,123 |
| ICSS 0.7.2, stock 128-page buckets | 8,193 / 512.06 | 966,683,123 |

All have 95,452,159 live payload bytes and identical checked contents. Storage
uses 41.8% more backing than the eight-page control but writes about 9.9 times
fewer bytes in this round. It still adds 12 pages through six growth calls in
that round; six rotations establish neither a leak nor a plateau. The directory
contains 2,332 extents. Physical high-water includes retained reusable capacity,
not just live values. [Compact results](../../measurements/storage-icss-reaudit.json)
retain all phases and provenance; the original same-size phase totals match.

No allocator replacement is promoted. Denser packing must retain selective
external-value writes, bounded checks and reuse safety. Raising INLINE globally
also increases routing traffic; a packed allocator adds metadata and changes
durable interpretation. Neither is justified by capacity savings alone.

### Checksum and directory hypotheses: no speculative production rewrite

Storage's payload checksums provide integrity checks absent from the compared
ICSS Cell/Vector representations. The retained bulk Cell replacement difference
scales at about eight instructions per extra byte. However, inspecting the actual
comparison Wasm shows that LLVM already unrolls the FNV loop eight bytes at a
time. Manual unrolling is not an established optimization. The serial FNV chain
and byte loads remain; changing the checksum algorithm changes stored bytes.
No checksum is weakened, no unsafe uninitialized read API is introduced, and no
2× gain is claimed. See [hash](../../../crates/ic-storage/src/hash.rs) and the
[retained measurement interpretation](../../local-memory-audit.md).

[Directory mutation](../../../crates/ic-storage/src/runtime/directory_model/persistence.rs)
verifies the persisted image, prepares a complete candidate and publishes it
before the root. Ordinary data IO does not repeat that work. ICSS's bucket growth
updates simpler metadata but does not provide the same named-grant/custody
contract. Interleaved growth can accumulate extents; this deserves a dedicated
IC growth-scaling experiment before reservation defaults or persistence change.
The current native study records extents and backing IO, not directory CPU cost.

## Validation and evidence

Rust/Cargo 1.99.1; locked offline graphs. Production and qualification lockfiles
retain their incoming selections. Focused B-tree tests cover borrowed decoding,
split/merge/reuse, failure publication, cache invalidation and the new histories.
Benchmark tests cover the host model, strict growth and bounded responses.

- `cargo test -p ic-storage --lib --locked --offline adapter::tree::tests -- --test-threads=1`:
  16 passed (`target/storage-memory-study/audit-tree-tests-2.log`).
- `cargo test --locked --offline --manifest-path testing/runtime-qualification/Cargo.toml -p ic-storage-benchmarks --lib --bin ic-storage-benchmarks --target-dir target/runtime-qualification`:
  12 passed (`audit-benchmark-tests-final.log` in the same directory).
- The shared-backing command in the compact JSON completed all eight backend/
  allocation-count runs; original phases compare exactly with retained results.
- Strict Clippy of the benchmark library, driver and optional-feature native
  example passed (`audit-clippy.log`). Selected Rust formatting, local report/
  changelog links and `git diff --check` passed.
- `make benchmark-storage BENCH_WORKLOAD=bulk BENCH_SAMPLES=1` passed at
  `target/storage-comparison/attempt.a6sTYA/`, including final source/artifact
  identity checks and unchanged lockfiles. A sandbox-only attempt could not bind
  PocketIC's listener; the explicitly approved local-listener run completed.
  The earlier 2 MiB response rejection remains at `attempt.tryvmc/`.

Evidence is under `target/storage-memory-study/audit-*`; failed attempts remain.
Installed comparison attempts retain source archives, both locks, compiler/tool
identities, Wasm hashes, complete replies and independent-model verification.
Native capacity results must not be described as IC instruction measurements.
No release, commits, dependency refresh, sibling edits or stored-format change.
