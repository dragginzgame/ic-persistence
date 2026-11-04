# Current handoff

## Published Storage 0.66.3 and Logging Storage 0.64.1

Storage `8504539` / `storage-v0.66.3` is pushed and published. All 12 release
stages and installed IO/upgrade/rollback qualification passed. The downloaded
registry archive matches the qualified SHA-256 `901edc46ec51eadacaac3f3413cba19cc0ec06b009c4f4f045b0eb6442988f37`.
Evidence: `target/storage-0663-deploy/`; runtime attempt `0K5mSN`.
Logging remains published at `74f185e` / `logging-storage-v0.64.1`; adapters were
not rereleased. Shared Tooling remains 0.29.4 (`628ccc7`, 153 files).
Preserve all failed logs, receipts and lock-preservation stashes, including
`storage-0663-preserved-pending-lock-inputs`. Restore its unrelated dependency
selections after publication while retaining Storage's new local version.

## Included in Storage 0.66.3

The ordinary matrix now covers 29 concrete cases and 52 type/size combinations.
The boundaries recipe covers 48 combinations across Map/Vector/Cell/Region;
sequential access also qualifies all four Set key types and bounded byte vectors.
Installed ICSS 0.7.2 comparisons passed 3,096 boundary and 1,008 sequential calls,
plus final-content checks. Fourteen native tests and strict benchmark Clippy pass.
One sample, three rounds, 64 rows (Cell one), eight-page ICSS buckets, disabled
map caches: these are scope-limited qualification runs, not a universal ranking.
[Retained summaries/configurations](../measurements/storage-icss-expansion-2026-11-04.json)
record source/Wasm digests; raw results remain under `target/storage-comparison/`.
Both incoming lockfiles are unchanged. The sandbox server-bind failure is retained
in `target/comparison-expansion/`; installed runs used authorized localhost access.
The [hotspot investigation](../audits/recurring/storage-icss-hotspots-2026-11-04.md)
finds checksum passes absent in ICSS Cell/Vector, existing reusable vector buffers,
and an aggregate-round interpretation caveat. Per-round summaries now distinguish
initial state from post-Shrink Cell reads. A subsequent borrowed-key cursor fix
passes 94 core tests and strict lint; matched scans improve 4–8% for array Map
keys and 8.45% for u64 Set keys (String Set 0.5%). Before/after runs pass 984 calls
each; Vector layout controls pass 672. Evidence: `target/cursor-investigation/`.
Key-only scans now also skip unused inline-value copies: another 7.76% reduction
in array-Map KeysScan (14.98% combined), with 0.3–0.6% Set scan increases versus
the first candidate. Final u64 Set scanning is 7.87% below the original baseline.
94 core tests, strict lint and 984 further installed calls pass. Evidence:
`target/key-only-investigation/`. Cell/Vector checks and formats remain.
The RSX-inspired inline replacement retains shared preparation: 3.74–7.47% fewer instructions for single replacements,
19.53–26.60% for batches of 32 in matched 256-row Uniform/Hot16 runs. All three
3,576-call traces pass; aggregate ICSS instructions and Storage pages match.
Last regresses 2.66%; Wasm grows 4,054 bytes, 56% less than the first candidate.
95 core tests/lint pass; evidence: hotspot audit and `target/encoded-update-size/`.
Experiment 2 remains research: external-value-only 64-byte blocks save 30–44%
of backing in the four-map small-record study, but Grow 33→132 costs up to 41%
more. Both prototypes pass 95 tests and 3,576 installed calls each. Production
format/minimum are restored; experiment 1 remains selected. Durable prototype
patches/results are linked in the hotspot audit. `--small-records` is retained
as a native benchmark selection; logs: `target/small-block-investigation/`.
Deeper qualification: four policies, 36 native 24-phase histories, 24 installed
runs / 39,168 calls; complete ICSS controls match. Retaining capacity with a
64-byte initial floor fixes recurring write churn and improves batch cycles,
but warm Uniform single-record 80-byte cycles regress 61.10%. No allocator
promotion: preserve baseline/experiment 1. Eight cache trials (12,288 calls)
identify protected-slot collisions at 18 KiB/64 KiB use. A mixer moves the failure
to baseline (+58.59%); rejected. Selected two-way cache removes the target collision
(−37.76%), with up to +2.04% warm-trace overhead; 9,408 new calls pass.
Core tests/lint (95/95/96) and 6+10+1 harness tests pass; both locks are unchanged.
Two-way cache adopted for 0.66.3; allocator unchanged. Sweep seeds/budgets next.
96 core/87 helper tests and all-target lint pass; CI helper lint warning fixed.

## Included CI group selection

Private adapter PRs now reuse the qualified release selector; non-docs pushes,
core/shared/public/unknown changes remain full. Admitted plans scope four Cargo
stages to the adapter, retaining workspace archives, policy and native/runtime
obligations. CI source identity, runner checkpoints and upload evidence remain
required. 87 helper tests, actual runner/Make scope fixtures, workflow/evidence
fixtures and both adapters' real lint/features/docs/Wasm checks pass locally.
See [the mapping](../storage-release-validation.md) and `target/ci-group-selection/`.
No hosted acceptance, release or measured total speedup is claimed.

## Published Metrics Storage 0.68.0

Metrics 0.29 adoption is published and unyanked: source `f99bceb`, release `7c71b60`,
tag `metrics-storage-v0.68.0` on origin/main. All 12 release stages passed; downloaded
registry bytes match the qualified archive. Evidence: `target/metrics-068-deploy/`.
Unrelated work is restored; preserve both `metrics-068-*` stashes and failed logs.

## Prior Metrics 0.28 and Wire 0.16 adoption

Storage 0.66.0, Metrics Storage 0.67.0 and Logging Storage 0.64.0 were published
with matching archives; historical identities and qualification paths remain in the
[delivered handoff](https://github.com/dragginzgame/ic-storage/blob/7c5cf83660ccefd98fb0947a011c99fdab464ef1/docs/status/current.md).
Do not repeat publication; Linux proof does not claim native macOS completion.
[Storage #154](https://github.com/dragginzgame/ic-storage/issues/154) is closed;
schema decoupling shipped in PR #155, supplier Observability #128 closed upstream.
Metrics Storage requires Metrics 0.28, Storage 0.66 and Wire 0.16; hosts align
public Rust identities. Backing/metric rows need no reset; Logging's migration
duties remain. Preserve `storage-0660-unselected-lock-edits`,
`storage-0660-preserved-pending-inputs`, prior failed attempts and tool caches.

## Published ICSS re-audit follow-through

Storage 0.66.2 includes borrowed entry decoding and corrected bulk/size-churn
benchmarks. The [re-audit](../audits/recurring/storage-icss-2026-11-03.md) preserves
scope, raw evidence and the distinction between published ICSS 0.7.2 and b5d1650.

## Prior publication and recovery

[Storage #153](https://github.com/dragginzgame/ic-storage/issues/153) is closed on
published Storage **0.65.5**, commit `127031ee`, tag `storage-v0.65.5`, and IcyDB's
registry consumer evidence. Its verified archive SHA-256 is
`4a5b1ea9a7b8f61e0ea22b257be5282795bd24ba77ab3d730afd02a46462b9aa`.
Prior full gate, archives and original/O3/Os/Oz installed IO/upgrade/rollback proof
remain in `target/storage-0655-deploy/`; do not repeat that release/upload.
[Its delivered handoff](https://github.com/dragginzgame/ic-storage/blob/ce52130d9e945aefee017faf905fc65e1db40cb3/docs/status/current.md) retains earlier qualification paths and interpretations.

Keep the `storage-0655-unselected-lock-edits`, `storage-0654-unselected-lock-edits`,
`storage-0653-unselected-lock-edits` and `storage-0653-hotspot-integration` stashes.
Earlier gate failures involved external lock edits and shared-target trybuild path
mismatches; their logs and final successful independent qualification remain intact.
Keep historical tool caches, recovery receipts and archives.

## Deferred memory and performance work

The [consumer audit](../local-memory-audit.md) records source inventories, not live
usage. Canic receipt bounds still need qualification against Storage; no capacity
failure or quantified saving is claimed. Role/feature counts are in the audit.

The explicitly authorized IcyDB layout preparation exists only in
`/tmp/icydb-control-layout`, branch `storage/control-allocation`, base `03c5713e`.
It is uncommitted: preserve it, do not clean or continue implementation there.
It separates schema controls, admits fresh images and rejects incompatible ones;
685 focused tests and native/Wasm lint passed, but installed composition is unqualified.
Approval remains pending to retain its complete tracked/untracked changes at
`/home/adam/projects/icydb-control-layout` or prepare an authorized draft PR there.
Do not change the normal sibling checkout or create an unsolicited PR.

The prior node-buffer study shows small integer-path improvements but a **12.74%
4 KiB batch-scan regression**, with total 4 KiB trace +0.15%; cause is unverified.
Cell checksum throughput and external-payload scans remain profiling candidates.
Matched measurements remain in `target/storage-node-buffers/` and
`target/storage-comparison/`; nine matched 1/8/128-page ICSS bucket comparisons
remain in `target/storage-bucket-study/`. These are collection-level measurements,
not whole-endpoint speedups or a universal Storage/ICSS ranking.

## Retained product boundaries

Storage owns allocation, custody and five collection forms; opaque IDs, host
lifecycle/framing and explicit fresh/reset backing remain. Map/Set/regions retain
`ICMAP002`; retired manager/collection images require explicit rebuild, never
an automatic reset or legacy decoder. Native failures poison ownership; IC traps
use rollback. Current rows are unchanged by this Metrics/Wire adoption.

Native production has no Runtime/Types. Default Wasm uses Runtime 0.19 IC0 only,
without executor, CDK, guard, Types or Candid. The independent qualifier uses
Platform/PocketIC 0.19.0, Testkit 0.79.0 and server 16.1.0. Platform 0.17 remains
only as the explicit ICSS 0.7.2 benchmark facade; pinned DFINITY Git `b5d1650` is
another research control. No sibling release or backing reset is authorized here.
