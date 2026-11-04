# Current handoff

## Published Storage 0.66.1

Storage **0.66.1** is published and unyanked. Release commit `06f64ca` and annotated
tag `storage-v0.66.1` were pushed atomically to `origin/main` from the approved
normal checkout. Source commit is `551a4aa`; [#156](https://github.com/dragginzgame/ic-storage/issues/156)
is closed. Metrics Storage is now 0.68.0; Logging Storage remains 0.64.0.
Downloaded registry bytes match receipt/archive SHA-256
`35d187e67fc27cc56f9ab6b3d9b125f6e8d73c538c55578888b27edd86dd8938`.
Publication verification and failed/successful logs: `target/storage-0661-deploy/`.

The source-bound full release gate passed all 12 stages; original proof is under
`target/release-plans/storage-1451973.1793709419076796644/`. Native validation log
is `target/release-validation/attempts/verify.Q9F6DG`; group receipts and exact
archives remain under `target/release-validation/`. Final archive integration
passed at `target/package-qualification/attempt.xewaoR/`. Hosted release CI was
running at handoff ([run](https://github.com/dragginzgame/ic-storage/actions/runs/41638040482));
local Linux proof does not claim macOS completion. Do not repeat release/uploads.

Prior Shared proof: `target/shared-tooling-0282-adoption/`, `target/shared-tooling-0281-adoption/`.
Preserve `storage-0661-preserved-unselected-lock` and `storage-0661-publication-unselected-lock`.
After delivery, unrelated Build/Host, Metrics 0.28.1 and Logging 0.24.4 locks were
restored; only Storage's local lock identity advanced to 0.66.1. Keep the evidence.

## Pending Storage 0.66.2 tooling and memory study

Shared **0.29.4** (`628ccc7`) is adopted from verified GitHub `main` via clean-source
export (153 files); no extra checkout or release was created.
The UTF-8 receipt regression and boundary/pin/format checks pass; logs are in
`target/shared-tooling-0292-adoption/`; the prior UTF-8 failure is retained too.
Current retention/caller evidence: `target/shared-tooling-0294-adoption/`. Seventeen
inline Rust test modules and private fixtures now use tests.rs/tests directories;
adapter test files are consolidated there too. Native/unit and actual CI-selection
checks qualify the relocation and classifier; no runtime format change. Logging's
test-only codec helper relocation is recorded separately for pending **0.64.1**.
Remaining consumer fixture defaults now use `target/fixtures` or
`target/repo-tool-fixtures`; retention/alias and native collision checks pass.
The active audit overlay references current Storage modules and Rust 1.99.1.
Focused evidence: `target/fixture-location-adoption/`; historical artifacts remain.
Rust/Cargo remain 1.99.1. Both incoming locks and pending benchmark work are preserved.

The [completed study](../local-memory-audit.md#completed-bucket-and-shared-backing-study)
extends 0.66.1's 93-row inventory and fixed `control|records|bulk` recipes.
[Results/provenance](../measurements/storage-memory-study.json): all nine matched
1/8/128-page ICSS 0.7.2 comparisons pass, **16,572 validated IC calls**. Frozen
driver/Wasm evidence remains in `target/storage-memory-study/` and
`target/storage-comparison/attempt.iLr6et/`. Eight pages becomes the research
default: records trace +0.76% instructions versus 128, 16 GiB bucket ceiling.
The new native 32/64-map shared-backing experiment passes growth/churn/value checks
and strict Clippy. At 64 maps: Storage 100.1875 MiB; ICSS 1/8/128 uses
56.0625/84.0625/512.0625 MiB. Storage writes far fewer bytes in this trace.
The fixed-shape probe confirms 20-byte framing/power-of-two space cliffs; prototype
finer external-value allocation next, retaining selective IO and corruption safety.
Production geometry/APIs and siblings are unchanged; no deployed/GiB claim.
Formatting, selected CLI test and ShellCheck pass. Prior recipe qualification and
failed fixture evidence remain under the earlier `target/storage-comparison/` attempts.

## Published Metrics Storage 0.68.0

Metrics 0.29 adoption is published and unyanked: source `f99bceb`, release `7c71b60`,
tag `metrics-storage-v0.68.0` on origin/main. All 12 release stages passed; downloaded
registry bytes match the qualified archive. Evidence: `target/metrics-068-deploy/`.
Unrelated work is restored; preserve both `metrics-068-*` stashes and failed logs.

## Prior Metrics 0.28 and Wire 0.16 adoption

Storage 0.66.0 (`0b10639`), Metrics Storage 0.67.0 (`ab333a1`) and Logging
Storage 0.64.0 (`cf23e04`) were published, with matching downloaded archives.
Evidence: `target/metrics-028-deploy/`, `target/release-validation/` and installed
original/O3/Os/Oz qualification `target/qualification/runtime/attempt.RFmb22/`.
Earlier full command/receipt identities remain in the
[delivered handoff](https://github.com/dragginzgame/ic-storage/blob/7c5cf83660ccefd98fb0947a011c99fdab464ef1/docs/status/current.md).
Do not repeat publication; Linux proof does not claim native macOS completion.
[Storage #154](https://github.com/dragginzgame/ic-storage/issues/154) is closed;
schema decoupling shipped in PR #155, supplier Observability #128 closed upstream.
Metrics Storage requires Metrics 0.28, Storage 0.66 and Wire 0.16; hosts align
public Rust identities. Backing/metric rows need no reset; Logging's migration
duties remain. Preserve `storage-0660-unselected-lock-edits`,
`storage-0660-preserved-pending-inputs`, prior failed attempts and tool caches.

## ICSS re-audit follow-through

The [re-audit](../audits/recurring/storage-icss-2026-11-03.md) distinguishes
published ICSS 0.7.2 from pinned DFINITY b5d1650. Pending 0.66.2 now borrows
map-entry encoded bytes for non-consuming decoding, with no format/API change.
Bulk Grow now changes size within the IC reply ceiling; variable-key/value
histories and six matched size-churn rounds extend the earlier coverage.
16 focused B-tree and 12 benchmark/model tests, native experiments and strict
benchmark Clippy pass. Corrected installed bulk: 172 measured calls and final
contents pass at `target/storage-comparison/attempt.a6sTYA/`; see the audit.
[Native results](../measurements/storage-icss-reaudit.json): at 64 allocations,
Storage uses 238.31 MiB versus ICSS eight-page 168.06 MiB, but writes about
9.9× fewer bytes in the final round. No plateau or leak claim follows from six
rounds. Prefer a measured denser-value prototype preserving selective writes;
directory scaling needs IC CPU measurements before redesign. FNV is already
eight-way unrolled in the measured Wasm; no speculative checksum change was made.

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
