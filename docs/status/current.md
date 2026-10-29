# Current handoff

Released source is Persistence 0.55.1 at `c3700a1`. Pending 0.55.2 adopts published
IC Platform 0.8.1. Own package versions remain 0.55.1; no commit, release,
publication, deployment or stable-data reset ran during this adoption.

## Selected Platform boundary

Both maintained catalogs select Platform 0.8.1 with their existing default-feature
and opt-in capability selections. Memory/Storage still use the exact registry
`ic-stable-structures` 0.7.2 package. Public substrate types, custody, allocations,
collection formats and both stable adapters' Wire records are unchanged.
Platform's raw Principal/runtime split needs no source repair: PocketIC supplies
external conversions for the runner; the Logging fixture keeps the raw CDK owner.

The root's production normal/build graph admits one Platform 0.8.1 identity.
Testkit Native 0.69.14 independently retains Platform 0.7.6 in native-only dev
support. Its stable-memory substrate still has the same exact package identity.
The independent qualification lock advances only Platform 0.7.3 to 0.8.1;
PocketIC client/server 16.1.0, tool/compiler pins and workspace scope are unchanged.

The incoming dirty root lock had already selected Build/Host/Testkit Native
0.69.14 and Metrics 0.17.3. Those selections are preserved. Initial offline
resolution briefly selected cached Testkit Native 0.69.13; explicit targeted
resolution restored 0.69.14 before qualification. No downgrade is retained.
Root Platform macros advance to 0.8.1 and unused thiserror 1 packages are pruned.

## Focused evidence

Strict Memory/Storage library/test Clippy with test support passes. Logging's
Wasm library/fixture Clippy and the independent runner's all-target Clippy pass.
All four runner unit tests pass; its two explicitly installed tests remain ignored.
Production graph/substrate identity and dependency pin checks pass.

All 12 stable-Cell tests and the governed collection reopen regression pass.
Formatting, local documentation links and diff checks pass. Session evidence is
retained under `/tmp/ic-persistence-platform-081.ueTdXi/`. No fresh
installed runtime, full release or native macOS qualification is claimed. The
unchanged substrate and client/server pair do not require a new matrix merely
because Platform's package version changed.

Incoming manifests/locks and the previous handoff are retained in that evidence
directory. Earlier release receipts, archives and qualification logs retain their
original source identities; do not reinterpret them as Platform 0.8 acceptance.
The four-package publication roster and required Logging receipt digest remain.
Metrics' existing old-row retirement contract remains in force; this Platform
adoption introduces no new reset requirement.
