# Current handoff

## Delivered Storage 0.65.2

Storage **0.65.2** is published and unyanked. Release commit
`eac8a84c6d5cf860f906b3fb788d8f275a1745b5` and annotated tag `storage-v0.65.2`
were atomically pushed to `origin/main`. The downloaded registry archive, fresh
sparse-index record and final receipt agree on SHA-256:
`127f3898bc06ed65478490f2f052ee5455c432b76ca286de95bcaaa46195fa0a`.
Do not repeat release/upload. See [qualification and provenance](../ic0-backing-qualification.md)
and [release notes](../../releases/storage/0.65.md).

This compatible patch requires published Runtime ^0.18.1 with defaults disabled,
only on Wasm, selecting `ic0` alone. All four stable64 backing operations use
Runtime's public safe wrappers. Growth maps `u64::MAX` to `-1` and checks successful
previous-page conversion. Native production has no Runtime dependency. Default
Wasm has no runtime, executor, CDK, guard, Types or Candid activation.

Fresh registry-only default consumers run on native and build on Wasm. Explicit
runtime composition also passes both targets, with one Runtime 0.18.1 and matching
guard 0.18.1. The registry graph resolved Wire/derive 0.15.1 and, for composition,
Platform 0.18.0 / Types 0.17.0. No paths, Git pins, registry patches or guard
overrides were used. This supplies Relay's requested Storage dependency;
[Relay #21](https://github.com/dragginzgame/ic-relay/issues/21) remains downstream
adoption. Relay and sibling checkouts were not edited.

## Qualification and checkout custody

Implementation commit `4db4f2c` passed independent default/composed graph checks,
consumer compilation/tests and the complete release gate in 230.067 seconds.
Final archive integration passed after version preparation and release commit.
Installed IO, growth/refusal, bounds, reopen, upgrades, rollback, directory faults,
backing custody, all five collection forms and Logging passed for original,
O3, Os and Oz Wasm with Testkit 0.79.0 / PocketIC 16.1.0.
Linux-local qualification does not establish native macOS execution.

Release work used `/tmp/ic-storage-ic0-release-0652` to exclude concurrent benchmark
edits in the original checkout. That isolated checkout retains
`target/storage-0652-deploy/`, final receipt
`target/release-validation/storage-0.65.2.json`, full gate `verify.S0HqOf`, final
archive attempt `QHiehT` and installed runtime attempt `VzJL5A`.
The [qualification record](../ic0-backing-qualification.md) maps complete paths.
Sandbox-denied attempts and earlier evidence remain retained. The original
checkout's benchmark work and local Git state were preserved.

## Independent groups and retained contracts

Metrics Storage **0.66.0** remains published (tag `metrics-storage-v0.66.0`, commit
`64777ae7a5ca916ecfcf7db94342b22008dc02c9`). Logging Storage **0.63.0** remains its
package version with **0.64.0** notes pending. Neither group was released or
published by this delivery. Storage's release updated only its own identity and
required local catalog/lock references; adapter versions and notes are unchanged.

Storage owns allocation, backing custody and Cell, Map, Set, Vector and map
regions. IDs remain opaque; hosts own grants, lifecycle, migration framing and
explicit reset/fresh backing. Map/Set/regions retain `ICMAP002` from 0.65.0.
Earlier AVL images require explicit rebuild/reset. No compatibility decoder or
automatic reset exists. Native backing failures poison the owner; IC traps retain
host rollback. See [SAFETY](../../SAFETY.md) and [allocation](../current-ledger.md).

Wire 0.15 owns current codec errors; Metrics Storage uses Metrics 0.27. Logging
checkpoints retain discriminator 3 and an explicit retained-text ceiling. Hosts
retire earlier rows explicitly. Storage 0.65.1's diagnostics, free-block reuse,
optional node caching, bounded scans and vector/extent IO improvements remain.

## Tooling, benchmark work and history

Shared Tooling 0.27.27 (`651f5f7`) remains the reviewed executable snapshot;
AGENTS' revision-bound policy remains authoritative. Rust 1.99.1 is the support
floor and pinned compiler. Build/metadata CLI 0.74.0, Host 0.74.1, Testkit CLI
0.79.0 and server 16.1.0 are unchanged. Both locks select Runtime 0.18.1 where
applicable; the independent qualifier's canonical guard is 0.18.1 and its native
client remains 0.18.0. No historical tool caches were removed.

Prior publication, comparison measurements and retained artifact paths remain in
the [0.65.1 handoff](https://github.com/dragginzgame/ic-storage/blob/a82de9b8682ddfff62a1cdaef09c189f229fd861/docs/status/current.md)
and [comparison provenance](../storage/upstreams.md). Benchmark results remain
mixed; large-record slack, directory snapshots and unreclaimed tails remain
costs. This release makes no superiority, GiB-scale or whole-call speed claim.
Concurrent benchmark changes are separate work, excluded from this delivery.
