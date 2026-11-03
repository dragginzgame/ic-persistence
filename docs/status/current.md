# Current handoff

## Published Metrics 0.28 and Wire 0.16 adoption

All three releases are published and unyanked. The normal checkout
`/home/adam/projects/ic-storage` on `main` performed the approved direct
`origin/main` and annotated-tag pushes, then separate crates.io publication:

| Package | Version | Release commit | Tag |
| --- | --- | --- | --- |
| Storage | 0.66.0 | `0b10639` | `storage-v0.66.0` |
| Metrics Storage | 0.67.0 | `ab333a1` | `metrics-storage-v0.67.0` |
| Logging Storage | 0.64.0 | `cf23e04` | `logging-storage-v0.64.0` |

Downloaded registry archives match each qualified receipt and index checksum.
Verification JSON, downloaded archives and release/publication logs are retained
in `target/metrics-028-deploy/`; exact group receipts and archives remain in
`target/release-validation/`. Do not repeat these releases or uploads.

Metrics Storage requires published Metrics **0.28**, Storage **0.66** and Wire
**0.16**. Metrics owns arithmetic/validated Serde with no Wire dependency;
Storage owns its record codec and services own DTO schemas. Public metric values,
errors and Wire limits change Rust identity, so hosts must align dependencies.
Current Storage-owned backing and metric rows require no reset or migration.
Logging's notes retain the distinct migration obligations from its older release.

[Storage #154](https://github.com/dragginzgame/ic-storage/issues/154) is closed with
supplier adoption, all 29 Metrics tests, strict native/Wasm lint, actual consumer
and production graphs, complete release gates and publication evidence. Its
schema-coupling fix was delivered in PR #155. Supplier Observability #128 is also
closed by its owner; no Storage issues remained open at final inspection.

All three complete local release gates and final archive integrations pass:
Storage `verify.qQYeeM` / `attempt.wRS8QW`, Metrics `verify.rOKp7A` /
`attempt.EEEctc`, Logging `verify.027t9u` / `attempt.hA7Wvv`.
Installed original/O3/Os/Oz IO, collection upgrades, custody, logging and rollback
pass in `target/qualification/runtime/attempt.RFmb22/`. Hosted/native-macOS CI
was pending or running at handoff; these Linux results do not claim it passed.

The delivered source includes reviewed Shared Tooling **0.27.29** (`18e7270`),
with its 145-file snapshot and Rust/Cargo 1.99.1 unchanged. Earlier focused proof
remains in `target/shared-tooling-02729-adoption/` and `target/wire-016-adoption/`.

Unrelated Build 0.74.1, Host 0.74.2 and Logging 0.24.3 lock selections are restored
as the normal checkout's sole pending source diff. Exactly those five package
blocks match stash `storage-0660-unselected-lock-edits`; all other lock blocks
match the released source. Keep that stash and `storage-0660-preserved-pending-inputs`.
The initial attempt refused dirty source before validation or version preparation;
its log remains. Subsequent clean releases passed without bypassing checks.

## Prior publication and recovery

[Storage #153](https://github.com/dragginzgame/ic-storage/issues/153) is closed on
published Storage **0.65.5**, commit `127031ee`, tag `storage-v0.65.5`, and IcyDB's
registry consumer evidence. Its verified archive SHA-256 is
`4a5b1ea9a7b8f61e0ea22b257be5282795bd24ba77ab3d730afd02a46462b9aa`.
Prior full gate, archives and original/O3/Os/Oz installed IO/upgrade/rollback proof
remain in `target/storage-0655-deploy/`; do not repeat that release/upload.
[Its delivered handoff](https://github.com/dragginzgame/ic-storage/blob/ce52130d9e945aefee017faf905fc65e1db40cb3/docs/status/current.md)
retains exact earlier qualification paths and benchmark interpretations.

Keep the `storage-0655-unselected-lock-edits`, `storage-0654-unselected-lock-edits`,
`storage-0653-unselected-lock-edits` and `storage-0653-hotspot-integration` stashes.
Earlier gate failures involved external lock edits and shared-target trybuild path
mismatches; their logs and final successful independent qualification remain intact.
No historical tool caches, recovery receipts or archives are cleanup candidates.

## Deferred memory and performance work

The [consumer audit](../local-memory-audit.md) records source inventories, not live
bucket usage: Canic 27 base plus five optional allocations, Blob 14, Relay three
per channel plus component stores, IcyDB `3 + 4J`. Canic receipt bounds still need
qualification against Storage; no capacity failure or quantified saving is claimed.

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
