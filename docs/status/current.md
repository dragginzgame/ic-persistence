# Current handoff

The repository implements its own stable-memory B-tree in `crates/ic-storage`.
It supports custom encoded-key order, inline small payloads, overflow chains,
page reuse, fallible operations, lazy bidirectional ranges and structural audit.
The `ic-stable-structures` dependency supplies the Memory interface only; its
collection implementation is not used. IC Memory remains the allocation owner.

Read [the IcyDB boundary](../design/icydb-btree.md) and
[the package contract](../../crates/ic-storage/README.md) before changing the
format, comparator, allocator or mutation publication behavior. No IcyDB
checkout changes or retained-data conversion have been performed. Native model
and failure tests and Wasm checks do not establish canister rollback/recovery.

Shared Tooling 0.15.4 is recorded in the root snapshot. Its revision
`879eb2b021522252a67e908dbc217a0f4efb2a6f` matches remote HEAD as checked on
2026-10-16; all 99 exported files pass snapshot verification. Uncommitted sibling
changes are not adopted. Development uses Rust 1.99.0; the library retains
Rust 1.85.0 support.

The released 0.1.1 patch makes value replacement reuse existing overflow pages
without splitting tree nodes. It preserves the public API and stored format.
Regression coverage includes full-root replacement, overflow/inline transitions,
reopening, page reuse and rejected growth without partial writes.

Additional 0.1.1 qualification exercises 700 length-framed keys against a decoded
key model through splits, mixed inline/overflow replacements, deletions, bounded
interleaved scans and reopening. The focused lifecycle test passes on Rust 1.99
and 1.85; the integration test target passes Clippy. No production change was
needed for this coverage. This remains a synthetic comparator, not IcyDB's codec.
The accepted IcyDB transition is a hard cut without a legacy reader; retained-data
disposition and canister qualification remain required before adoption.

Cargo.toml and Cargo.lock select released 0.1.4, tagged at `60d0f01` (`v0.1.4`).
Current qualification uses IC Host FS 0.22.1 and IC Metrics 0.11.5 from that
release's lockfile. Release metadata is untouched during implementation.

Released 0.1.2 reuses the located node and reads the previous overflow value once
during replacement. Native interface read traffic is halved for the index,
journal-sized value and row fixtures; writes and reserved capacity are unchanged.
The reproducible command and before/after measurements are in the
[IcyDB design](../design/icydb-btree.md#native-memory-traffic-qualification).
The API, persisted format and staged-write failure contract remain unchanged.
Validation: 16 storage tests pass on Rust 1.99 and 1.85; the opt-in traffic
fixture, Clippy, Wasm compilation, rustdoc, formatting and doc links also pass.
This does not establish canister instruction costs, heap usage or IcyDB integration.
Release versions, commits, tags and publication remain maintainer-owned.

Direct upstream qualification now compares both persisted trees with a heap
model: 8,000 regular and 320,000 extended seeded operations passed, plus boundary
ranges, maximum payloads and rejection of actual upstream-format memory. The
four regular tests pass on Rust 1.99 and 1.85; the extended campaign passed in
release mode on 1.99. Traffic fixtures compare unbounded upstream storage and a
bounded index layout. Small index entries cost more bytes/capacity in our tree,
while large-value replacements use less traffic than the unbounded upstream
fixture. Fewer Memory calls do not prove fewer IC instructions; actual IcyDB
schemas and bounded large-value comparisons remain unqualified. See the design
document for reproducible commands, measurements and limitations.

Released 0.1.3 adds profiles using IcyDB's bound metadata from committed revision
`e104fe1c9f5921336f0e5b292acbda99f8311476`: variable index keys up to 16,668
bytes with variable one-byte values; fixed 12-byte journal keys with variable
64 KiB chunks; variable data keys up to 262 bytes with unbounded row values.
Synthetic bytes preserve sizes/order but do not implement the actual codecs.
The seven-profile comparison passes on native Linux; broader index bounds make
upstream capacity much larger than the earlier eight-byte-bound baseline.
Our small-entry read/write byte costs remain higher. See the design document's
bound-matched results before extrapolating from either baseline.

The 0.1.3 continuation extends those profiles through shrink/regrowth and
delete/refill phases, auditing and reopening between phases. All seven profiles
reuse our existing capacity. It also removes a redundant read during overflow
reclamation: `free` returns its validated prior page to chain traversal. The
new traffic regression fails before the fix and passes afterward. A 4 MiB row
removal now reads 8,454,144 bytes instead of 12,677,120. No API, persisted format
or release metadata changes are made.
Validation: all 18 regular storage tests pass on Rust 1.99 and 1.85; the four
regular upstream tests and 320,000-operation extended campaign pass on 1.99.
The seven churn profiles, Clippy, Wasm compilation, formatting and doc links pass.

`ic-storage-bench` now supplies a non-published canister and native runner using
IC Testkit 0.48.0, IC CDK 0.20.3 and IC Host FS 0.21.0. Its Rust 1.99 floor is
separate from the public library's Rust 1.85 floor. `make bench-run` accepts an
explicit pre-admitted server; regular CI compiles the native and Wasm targets,
while simulator execution remains opt-in. No server is downloaded implicitly.

Two fresh runs per engine/profile completed with PocketIC server 16.0.0 and
client 16.1.0 on Linux. Each operation's instruction count repeated exactly.
The measured results change the traffic-only conclusion: our key-only scans
use fewer instructions in these fixtures, but large-row reads and mutations
use more. A 4 MiB replacement costs 215,655,911 instructions here versus
124,318,147 upstream. See the design document for all results, artifact hashes,
measurement exclusions and retained failed/successful attempts. These synthetic
canister results do not qualify actual IcyDB codecs, production subnet costs,
rollback or upgrade recovery. No dependency was added to the library itself.

Instruction investigation retains two compatible optimizations: checksum
iteration in eight-byte chunks with the same FNV-1a byte order, and boxed page
buffers in the transaction staging index. The former reduces loop overhead;
the latter avoids moving 4 KiB buffers when the heap B-tree balances. Combined,
4 MiB replacement falls from 215,655,911 to 146,834,753 instructions and removal
from 265,860,686 to 195,445,266. Total Wasm heap capacity after replacement falls
from 17,367,040 to 13,893,632 bytes. Large-row operations still trail upstream.
Two fresh runs repeat all counters exactly; the design records intermediate
measurements and artifact identity. API, persisted bytes and staging semantics
are unchanged. Two new checksum tests compare the original persisted algorithm
and reject corruption at every page byte. These and all 18 regular storage
tests pass on Rust 1.99 and 1.85; four regular upstream comparisons, the extended
320,000-operation campaign, and native/Wasm Clippy pass on 1.99. Formatting and
documentation links pass. Full release validation and macOS remain outside this run.

The canister runner now covers 15 fixtures and ten ordered operations, with
shared host/canister descriptors. It exercises inline and overflow boundaries,
long keys, tree sizes, misses, key/value scans, shrink/regrow, removal/refill and
new insertions. Each measured build produces 600 rows across two fresh runs;
all counters and result lengths repeat exactly. Bulk value collection replaces
byte-by-byte flattening in the scan harness; earlier scan evidence is retained.

New free pages compute the identical FNV-1a checksum of their known-zero tail
using a compile-time multiplier. Existing pages still verify every byte.
Expanded 4 MiB removal falls from 195,445,228 to 155,867,505 instructions;
shrink falls from 196,375,461 to 156,797,740. Both still trail upstream. Mid-size
rows can win, while 17-byte values and long keys expose substantial losses;
value scans must not be conflated with key-only scans. See the design for the
full matrix, reproducible commands, hashes and measurement limitations.
Three checksum tests and 18 regular storage tests pass on Rust 1.99 and 1.85;
four regular upstream tests and the 320,000-operation campaign pass on 1.99.
Native and Wasm Clippy pass. No public API or persisted format change is made.

Released 0.1.4 folds verified trailing zero chunks into the checksum using the
same FNV-1a arithmetic. It still inspects all covered bytes; full nonzero payload
pages pay one failed suffix check. The format and integrity guarantees are
unchanged. A new test compares the original checksum at all 4,088 possible
last-nonzero-byte positions, alongside existing full-page corruption tests.

The runner now tests both zero-filled and nonzero-filled key suffixes, producing
1,200 measurements per build. Fresh baseline/optimized builds use the same
working dependency selections. All 300 operation/profile/pattern comparisons
improve by 0.67–53.13%, and each build repeats every counter/result length exactly.
With nonzero keys, 17-byte reads improve 37.6% and value scans 52.3%; maximum-key
lookups improve only 9.4%, versus 42% with zero-filled keys. Small-value mutation
and large-row costs still exceed upstream. Full measurements and artifact
identities are in the design document; these remain synthetic canister fixtures.
Four checksum tests and 18 storage tests pass on Rust 1.99 and 1.85. Four regular
upstream tests, the 320,000-operation campaign and native/Wasm Clippy pass on 1.99.

The 0.1.4 continuation collects the returned value while reclaiming its overflow
pages. The preliminary existence lookup reads keys only, removing a second
payload traversal. Nonzero-key 4 MiB removal falls from 154,691,671 to
106,900,172 instructions versus upstream's 120,839,997; 1 MiB removal also becomes
cheaper than upstream. The 16-byte inline fixture regresses about 0.12%, and
small overflow values remain expensive. All 1,200 matrix measurements repeat
exactly. The tightened read-budget test fails before the change and passes
afterward. A new late-payload corruption test proves no writes, unchanged bytes
and length, and successful removal after repairing only the injected corruption.
Four checksum and 19 storage tests pass on Rust 1.99 and 1.85; four upstream
tests, the 320,000-operation campaign and native/Wasm Clippy pass on 1.99.
API, stored format and release metadata are unchanged.

Pending 0.1.5 removes the second payload read during shrink. The old chain is
fully validated and collected first, then excess pages are staged free in the
same reverse order as before. Ordinary freeing still reads/checks the live page;
the new staging helper is private to callers that already performed validation.
The read-budget test fails before the fix and passes for inline and overflow
shrink with growth refused, including returned data and reopening checks.

The runner adds changed-byte replacement, asserting the old return and new
stored bytes, then restoring common inputs outside measurement. Both fresh
builds produce 1,320 measurements with exact repeat counts. Nonzero-key 4 MiB
shrink falls from 155,648,170 to 102,600,261 instructions versus 120,853,461
upstream. Before the encoder optimization below, changed-byte same-size
replacement remained about 146M versus 124M.
Four checksum and 20 storage tests pass on Rust 1.99 and 1.85; four upstream
tests, the 320,000-operation campaign and native/Wasm Clippy pass on 1.99.
No API, format, allocation ownership or package version changes are made.

The 0.1.5 continuation shares one heap-buffer encoder for new and replacement
overflow pages. It writes the header and payload directly, then zero-fills only
the remaining tail before sealing. Byte-for-byte tests cover empty, partial and
full payloads and pointer extremes. Nonzero-key 4 MiB changed-value replacement
falls from 145,687,077 to 132,826,728 instructions, still above upstream's
124,157,534. New insertion falls from 79,478,225 to 66,626,558. Both intermediate
and final builds repeat all 1,320 measurements exactly; no instruction increases
appear against the prior matrix. Small-value overhead remains significant.
Five codec/checksum tests and 20 storage tests pass on Rust 1.99 and 1.85;
four upstream tests, the 320,000-operation campaign and native/Wasm Clippy pass
on 1.99. No unsafe code, dependencies, format or API changes were introduced.

The next 0.1.5 optimization skips unchanged replacement-node and header writes.
It still validates the full old payload before staging replacement bytes;
changed metadata remains published under the existing failure contract.
Nonzero-key 17-byte changed replacement falls from 655,289 to 278,828 instructions
(still 1.46 times upstream), and 4,072-byte replacement falls from 713,329 to
336,868 (0.68 times upstream). The matrix now beats upstream in 232/330 cases;
126 cases increase against the prior build by at most 0.036%. Both fresh runs
repeat all 1,320 measurements exactly; artifacts and hashes are in the design.
The new write-budget regression fails before the fix and passes afterward.
Five codec and 21 storage tests pass on Rust 1.99 and 1.85; four upstream tests,
the 320,000-operation campaign and native/Wasm Clippy pass. Formatting and links
pass. API, persisted bytes, dependency selections and release metadata are
unchanged; large-row reads/value scans and small-value mutations still need work.

A further 0.1.5 checksum optimization inspects verified zero tails in 32-byte
blocks with an initial eight-byte guard for dense payloads. All covered bytes
are still checked and the persisted checksum is identical. The unguarded
experiment regressed large dense reads and was superseded; both attempts are
retained in the design evidence. The guarded version improves all 330 measured
cases against the prior build, up to 6.10%; nonzero-key 17-byte reads improve
3.08% and value scans 5.77%. Dense 4 MiB gains are negligible. All 1,320 rows
repeat exactly. Five codec and 21 storage tests pass on Rust 1.99 and 1.85;
four upstream tests, the 320,000-operation campaign, native/Wasm Clippy,
formatting and doc links pass. No API, format, dependency or version changes.

The expanded 0.1.5 matrix now has 23 fixtures: eight additions cover 31/32-entry
root transitions, 32/33-byte keys, and 32/256/1,024/8,192-byte rows. Each build
produces 2,024 rows. An inline-key allocation optimization was rejected after
the new 8 KiB value scan exposed a 14.90% regression and higher heap capacity.
The retained insertion change reuses the already-validated root and child nodes,
refreshing after splits. All 92 insertion/refill cases improve, up to 17.55%;
the other 414 counts and upstream counts are unchanged. All three measured
builds repeat exactly (6,072 rows). The new read-budget test fails before the
fix and passes afterward, including branch descent, reopening and validation.
Five codec and 22 storage tests pass on Rust 1.99 and 1.85; four upstream tests,
the 320,000-operation campaign, native/Wasm Clippy, formatting and links pass.
New cases confirm that small overflow rows and 33-byte keys remain expensive.
Evidence and artifacts are in the design; API, format and release metadata
remain unchanged. No publication or IcyDB integration was performed.
