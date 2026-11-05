# Current handoff

## Published on 2026-11-05

| Package | Version | Release commit / tag |
| --- | --- | --- |
| Storage | 0.67.1 | `e89f222` / `storage-v0.67.1` |
| Metrics Storage | 0.69.0 | `5f4dbdd` / `metrics-storage-v0.69.0` |
| Logging Storage | 0.65.0 | `a90632c` / `logging-storage-v0.65.0` |

All three releases are pushed to origin/main, published and unyanked on crates.io.
Each passed all 12 release stages and final archive qualification. Downloaded
registry archives match the qualified SHA-256 values:

- Storage: `0621f56cfcc42a109d1d52ac39410c0b7d7c7d3bbb9f46274c2381dced2e9352`.
- Metrics: `9429bc9b05c3eb251c201857c721157ddcfa88e6ba47c72f1e0aee9b58c02df8`.
- Logging: `67b3b84edaf5c180c6268b68af0a97e6e02116272ce0753b6bf88d240e79a045`.

Evidence: `target/tooling-030-deploy/` and group-bound receipts under
`target/release-validation/`. Preserve failed attempts and their retained logs.
Release qualification caught and corrected shared metadata ordering, fixture
copying and recovery assertions. Storage and Logging now project their respective
local identities into the independent fixture catalog/lock through Shared's
`shared_metadata_files`; Metrics preserves those files.

Shared Tooling 0.30.0 (`b37a882`, 154 files) is adopted. The
[adoption audit](../audits/2026-11-05-shared-tooling.md) retains the directory,
fixture and scoped-lint changes plus installed original-Wasm IO/upgrade/rollback
proof. Both LOC reports classify every source. No new native macOS proof is claimed.
All packages use Wire 0.17 without metadata derive; hosts exchanging public Wire
limits/errors must align that dependency. Codec bytes and persistent layouts are
unchanged; no backing reset is required. Neither adapter release remains pending.

Unrelated root-lock supplier selections are preserved in
`storage-tooling-030-preserved-pending-lock-inputs` and restored after deployment.
Keep that stash and all earlier preservation stashes, including
`storage-0670-preserved-pending-lock-inputs`; no historical evidence was deleted.
Earlier Storage 0.67.0 and Wire-adoption evidence remains in
`target/storage-0670-deploy/` and `target/wire-017-adoption/`.

## Included in Storage 0.66.3

The [ICSS hotspot audit](../audits/recurring/storage-icss-hotspots-2026-11-04.md)
retains the 29-case/52-combination matrix, cursor and inline replacement gains,
and rejected allocator/cache experiments. The two-way cache and inline update
are selected; 64-byte allocator prototypes remain research. Preserve their
tracked patches and `target/cursor-investigation/`, `target/key-only-investigation/`,
`target/encoded-update-size/` and `target/small-block-investigation/` evidence.
These are collection-level comparisons, not whole-endpoint throughput claims.

## Included CI group selection

Private adapter PRs now reuse the qualified release selector; non-docs pushes,
core/shared/public/unknown changes remain full. Admitted plans scope four Cargo
stages to the adapter, retaining workspace archives, policy and native/runtime
obligations. CI source identity, runner checkpoints and upload evidence remain
required. 87 helper tests, actual runner/Make scope fixtures, workflow/evidence
fixtures and both adapters' real lint/features/docs/Wasm checks pass locally.
See [the mapping](../storage-release-validation.md) and `target/ci-group-selection/`.
No hosted acceptance, release or measured total speedup is claimed.

## Previous Metrics Storage 0.68.0

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
