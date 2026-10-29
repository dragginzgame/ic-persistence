# Current handoff

Released source is Persistence 0.55.0 at `1ff8f93`. Pending 0.55.1 integrates
[PR #138 — Qualify stable Logging rollback and upgrade behavior](https://github.com/dragginzgame/ic-persistence/pull/138)
and [PR #139 — Enable coordinated publication of IC Logging Stable](https://github.com/dragginzgame/ic-persistence/pull/139).
Own versions remain 0.55.0. No release, registry upload, application deployment or
actual stable-data deletion ran. Source adoption does not publish Logging.

## Accepted source and recovery boundaries

PR #138 merged at `6a6c86b6`. The publication branch is reconciled normally with
that main. Both production Wire codecs remain byte-identical to released 0.55.0.
PR #139 changes no runtime source or selected lockfiles relative to merged #138.
The fixture additions preserve every original registry identity. Root Wire keeps
default features disabled; the independent graph retains PocketIC 16.1.0.

One canonical four-package roster owns workspace projection, bootstrap, archive
qualification, receipts and dependency-ordered publication. The old unpublished
side roster is removed. Current prepared/final receipts require `logging_sha256`.
Older releases and receipts must use their original source/tooling; no conversion
or missing-field fallback is maintained. Logging uploads last; all four registry
identities are observed before any upload effect.

Metrics' approved hard cut still requires explicitly retiring old metrics rows or
supplying fresh governed backing and advancing reset epochs. No legacy decoder
or automatic reset exists. Never clear Memory's allocator or unrelated collections.
[Upgrade disposition](../../crates/ic-metrics-stable/README.md#upgrade-disposition).

## Focused evidence on reconciled source

- All 68 release-helper regressions and strict helper Clippy pass, including
  mandatory Logging receipts, missing/corrupt archive refusal before registry
  observation, rejected final upload and lost-response reconciliation without
  replaying earlier package uploads.
- Four exact development archives qualify offline; the extracted consumer runs
  on Rust 1.99.1 and 1.91 and checks for Wasm. Source/archive hashes stay unchanged.
  These `--allow-dirty` development archives are not publication receipts.
- Actual archive admission/substitution refusals, native compiler dispatch,
  manifest-first/lock-first bootstrap and release-adapter substitutes pass.
- Selected ShellCheck with source companions, pins, production graph, doc links,
  formatting and diff checks pass. Bare ShellCheck lacked source companions; the
  corrected documented selection passes. The first release-adapter attempt lacked
  the isolated host-tool cache; it is retained, and the prepared-cache rerun passes.
- #138's original-Wasm installed proof passes on the current 0.55 Wire source:
  query non-persistence, trapped update/upgrade rollback, retained reopen,
  timestamp refusal and empty-history sequence/time watermarks. Fixture/library
  Wasm Clippy, independent runner Clippy and focused protocol regressions pass.

Archive evidence: `/tmp/ic-persistence-pr-adoption.2mnEQI/target/package-qualification/attempt.HJWViI/`.
Installed evidence: `/tmp/ic-persistence-pr-adoption.2mnEQI/target/qualification/runtime/attempt.FjZ2Kp/`.
Wasm SHA-256: `eec5e928d560a998e03113ec1b8fccefddb8f6d5c6296285df730cf2f7156399`.
Other logs/failed attempts: `/tmp/pr138-*`, `/tmp/pr139-*` and retained fixture paths
in those logs. The local Testkit-owned server ran only for the focused proof.

Updated hosted CI runs asynchronously and retains its exact PR-head identity.
Historical complete Linux/macOS original/O3/Os/Oz acceptance belongs to `1928117a`;
it is not relabelled as new merged-source qualification. No fresh full release,
macOS/optimized matrix, live Canic/Jobs acceptance or cost measurement is claimed.

Prior handoffs and codec evidence remain in `/tmp/pr139-retained-previous-handoff.md`,
`/tmp/pr138-retained-previous-handoff.md`, `/tmp/ic-persistence-before-metrics-wire-handoff.md`
and `/tmp/ic-persistence-logging-wire.AFMlbH/`. Retain all earlier journals,
receipts and archives; next release/publication uses the new four-package contract.
