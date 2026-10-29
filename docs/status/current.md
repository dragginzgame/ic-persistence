# Current handoff

Released source is Persistence 0.55.0 at `1ff8f93`. Pending 0.55.1 adopts
[PR #138 — Qualify stable Logging rollback and upgrade behavior](https://github.com/dragginzgame/ic-persistence/pull/138).
Own package versions remain 0.55.0. No release, publication, deployment or actual
stable-data deletion is selected. PR #139 separately prepares Logging publication.

## Reconciled source

PR #138 is merged with released main in an isolated checkout. Production Logging
and Metrics Wire codecs are byte-identical to released 0.55.0. Existing root lock
package identities are all preserved; eleven additions support the Wasm fixture.
The existing root Wire catalog selection retains default features disabled.
The independent workspace keeps its own lock/catalog and PocketIC 16.1.0 pair.

Metrics' approved hard cut remains: retire all old metrics rows explicitly before
adoption or supply fresh governed backing; no legacy decoder or automatic reset.
Never clear the allocator or unrelated collections to retire metrics.
[Upgrade disposition](../../crates/ic-metrics-stable/README.md#upgrade-disposition).
Logging uses its released Wire checkpoint format; earlier experimental bytes
remain retired. Logging is still unpublished in this PR.

## Focused evidence on reconciled 0.55 source

- Strict Logging library/fixture Wasm Clippy, release-helper Clippy and independent
  runner all-target Clippy pass on Rust 1.99.1 with locked offline dependencies.
- The two affected release-helper regressions and the shared fixture protocol
  maximum-row/method-identity regression pass.
- Pins, production graph, documentation links, formatting and diff checks pass.
- The focused original-Wasm installed Logging proof passes with prepared Testkit
  CLI 0.69.11 and PocketIC 16.1.0: query non-persistence, trapped update/upgrade
  rollback, retained reopen, timestamp refusal and empty-history watermarks.

Evidence is retained in `/tmp/ic-persistence-pr-adoption.2mnEQI/target/qualification/runtime/attempt.FjZ2Kp/`.
Wasm SHA-256: `eec5e928d560a998e03113ec1b8fccefddb8f6d5c6296285df730cf2f7156399`.
Focused logs: `/tmp/pr138-*`. The first runtime dispatch used a missing pin-file
path and stopped before launch; its failed evidence remains retained. The corrected
canonical pin path passed. The local Testkit-owned server ran only for this proof.

Original PR head `1928117a` passed its complete hosted CI, including Linux and
both macOS runtime variants. That evidence retains its original source identity;
new merged-source hosted CI is separate. No fresh complete release, optimized
matrix, macOS, Canic/Jobs installed acceptance or cost measurement is claimed.

Previous detailed handoffs and Metrics/Logging evidence remain at
`/tmp/pr138-retained-previous-handoff.md`, `/tmp/ic-persistence-before-metrics-wire-handoff.md`
and `/tmp/ic-persistence-logging-wire.AFMlbH/`. Earlier retained receipts, release
attempts and archives must use their original tooling; do not relabel their proof.
