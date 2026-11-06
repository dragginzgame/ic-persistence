# Current handoff

**Shared Tooling refresh (unreleased):** adopted 0.30.5 at `be37710`, the checked
GitHub `main` revision, through the canonical exporter. All 156 payloads match
the independently selected commit. The Candid source gate covers 161 Rust files;
both workspaces reject named direct CBOR dependencies. Actual formatter/LOC
callers, dependency pins/boundaries, a selected native release-note caller test
and the benchmark output-route/refusal probe pass. Source review found no CBOR
use; general CBOR import/macro/type-resolution enforcement is not supplied by
the lexical Candid check. New raw comparisons use `target/evidence/`.
Both lockfiles and 124 previously qualified Rust/manifest inputs are unchanged.
Details are in [the adoption boundaries](../shared-tooling-adoption.md). The
authorized measurement cleanup is complete; compact results replace bulk exports.
Engine work, stashes and release/recovery artifacts remain preserved. Logs are
under `target/evidence/shared-tooling-0305/`; no release or sibling edit occurred.

**Completed:** the full ICSS + Storage type union now covers all five supported
collection forms: 73 concrete types, 565 unique type/role cases and the full ICSS
B-tree key/value grid. All 46,604 installed measured calls and final states pass,
alongside 42 focused native tests and strict native/Wasm lint. The compatibility
ledger explicitly records 53 verified runtime refusals and 18 non-Ord backend
roles; absent ICSS measurements have blank costs/ratios. See the
[full-catalogue record and results](../audits/recurring/benchmark-full-catalogue-2026-11-05.md).
`BENCH_WORKLOAD=types make benchmark-storage` runs five automatic collection
shards with a common eight-MiB synthetic benchmark stack and 30-minute server
limits. Production stack settings and codecs are unchanged by this follow-up.
Raw evidence is in `target/storage-comparison/attempt.tSHDxo/`; partial/failed
attempts remain preserved. Previous 58-case evidence below is historical.

## Pending Storage 0.68.0: copied ICSS collection baseline

The maintainer selected all engines from copied published ICSS 0.7.2 source,
with governed allocation, successful-call API shapes and ICSS trap behavior.
Private `engine` contains algorithms; public `adapter` enforces checked leases.
No production ICSS dependency or copied MemoryManager exists. Old tree/block/
record implementations, node cache and Vector windows/epochs are deleted.
[Provenance](../storage/upstreams.md) and [qualification/results](../audits/recurring/icss-engine-baseline-2026-11-05.md)
record deliberate adaptations and original source hashes/notices.

Every earlier Storage collection requires explicit rebuild/reset or fresh
allocation; BTR2 (explicit page size), SVC1 and SCL1 are current. No legacy reader,
migration or automatic reset exists. Directory bytes/checksums and application
codecs are unchanged. Collection codecs/engine failures trap; collection
checksums are gone. Native mutation unwinds poison the shared owner; IC rollback
supplies Wasm atomicity. Positive cache ceilings do not enable a node cache.

Core collection/governance, custody, Metrics/Logging and reference/model tests
pass, with native/Wasm lint and production-graph checks. Original final Wasm
installed proof covers 1,031 allocations, all collections, successful upgrades
and decoder/update/upgrade rollback. Counts and evidence are in the audit.

The 3,196-call comparison plus 824 iterator reruns establish the reset baseline;
the 480-call Auth preset also passes. No universal speedup is claimed. These use
one ordinary seed/uniform access and ICSS eight-page research buckets. Evidence
is in `target/icss-baseline-reset/` and attempts `difIFu`, `UZTbp1`, `Q4kJp3`.

Changes are uncommitted/unpublished; manifests retain released versions.
No release, sibling migration or backing reset is authorized. The independent
lock preserves every selected package version/source and adds only the benchmark
package's existing `ic_principal` dependency edge. Root supplier refreshes beyond the starting snapshot
remain intact; root checks use Wire 0.17.2 / Runtime 0.19.2 / Build-Host 0.74.4,
while the independent comparison retains Wire 0.17.0 / Runtime 0.19.0.
Full release/packaging and optimized-Wasm qualification have not been run.

## Published on 2026-11-05

| Package | Version | Release commit / tag |
| --- | --- | --- |
| Storage | 0.67.1 | `e89f222` / `storage-v0.67.1` |
| Metrics Storage | 0.69.0 | `5f4dbdd` / `metrics-storage-v0.69.0` |
| Logging Storage | 0.65.0 | `a90632c` / `logging-storage-v0.65.0` |

All three are pushed and published, with 12-stage gates and downloaded archive
checksum verification. [The delivered handoff](https://github.com/dragginzgame/ic-storage/blob/4dea092/docs/status/current.md)
retains full hashes, prior release identities and recovery evidence. Current
release evidence remains in `target/tooling-030-deploy/` and group receipts under
`target/release-validation/`. Do not repeat these publications.

Those publications included Shared Tooling 0.30.0 (`b37a882`, 154 files).
All packages use Wire 0.17;
hosts exchanging public Wire limits/errors must align that identity. Codec bytes
and stored layouts are unchanged. The [adoption audit](../audits/2026-11-05-shared-tooling.md)
retains installed original-Wasm IO/upgrade/rollback proof; no macOS rerun is claimed.
Storage/Logging project their own identities into the independent catalog/lock
through shared release metadata; Metrics preserves those files.

## Historical evidence and preservation

The 2026-11-06 authorized cleanup retired 69 bulk/repeated experimental data and
prototype files and consolidated five superseded reports. The
[measurement index](../measurements/README.md) links current summaries and selected
results. All 565 cases retain backend status and selected Get/mutation costs;
complete per-run outputs are no longer tracked documentation. Historical figures
are not measurements of the current copied engines. Existing target run directories
were left untouched but remain removable by explicit cargo clean; no permanent
raw archive is promised and a digest cannot verify missing raw data.

Source, release/recovery records, small qualification tables and consumer inventory
remain intact. No cache, stash or target-tree cleanup was performed. Old collection
formats/prototypes are retired, rather than maintained as alternate implementations.
CI group qualification remains in `target/ci-group-selection/` and
[the validation mapping](../storage-release-validation.md).

Preserve `storage-tooling-030-preserved-pending-lock-inputs`,
`storage-0670-preserved-pending-lock-inputs`, all `metrics-068-*` stashes,
`storage-0660-unselected-lock-edits`, `storage-0660-preserved-pending-inputs`,
`storage-0655-unselected-lock-edits`, `storage-0654-unselected-lock-edits`,
`storage-0653-unselected-lock-edits`, `storage-0653-hotspot-integration`, all other
preservation stashes, failed attempts, historical caches, receipts and archives.
Closed #153/#154 and publication evidence remain linked by the delivered handoff.

## Deferred work and boundaries

The [consumer inventory](../local-memory-audit.md) lists declarations, not live
usage. Canic receipt bounds and complete Auth endpoints remain unqualified.
Any adoption must explicitly replace old collection images and host framing.
No current public API raw-backing extraction or independent ID assignment is added.

The authorized IcyDB layout preparation exists only in `/tmp/icydb-control-layout`,
branch `storage/control-allocation`, base `03c5713e`. Preserve its complete
uncommitted tracked/untracked work; do not clean or continue implementation there.
Its 685 focused tests and native/Wasm lint passed; installed composition is unqualified.
Approval is pending for durable retention at `/home/adam/projects/icydb-control-layout`
or an authorized draft PR. Do not modify the sibling or create an unsolicited PR.

Native production has no Runtime/Types. Wasm selects Runtime IC0 only, without
executor, CDK, guard, Types or Candid. Independent qualification uses Platform/
PocketIC 0.19.0, Testkit 0.79.0 and server 16.1.0. Platform 0.17 remains only as
the explicit ICSS 0.7.2 benchmark facade; pinned Git `b5d1650` is a research control.
