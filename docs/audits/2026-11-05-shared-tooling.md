# Shared Tooling 0.30.0 adoption audit

Reviewed 2026-11-05 against committed `07dce12cbe4a798fd4a6d263095518678c4f6b45`.
This is a repository hygiene and tooling audit, not a new performance, security,
retained-format or fleet compatibility qualification. Siblings remained read-only.

## Corrections

- The canonical exporter adopted 154 shared files without local vendor patches.
  The source checkout was clean and its commit matched GitHub `main`.
- Development utilities now live in `crates/ic-storage/tools/`; the `repo-tool`
  Cargo target and commands retain their names. Size probes live in package
  `tests/fixtures/`, and comparison targets/helpers in `benches/`.
- Installed canisters and their three passive Wire protocols now have one private
  owner: `testing/runtime-qualification/crates/ic-storage-test-fixtures/`.
  The native runner imports those protocols through a declared dependency.
  Three cross-directory Rust `include!` module substitutions were removed.
  Canisters, protocols and the native runner stay outside the root release graph.
  The existing independent workspace owns the same `wasm-size` profile; Make
  selects the explicit package/manifest and preserves artifact paths.
- Both workspaces declare LOC roles for fixture/benchmark-only packages. Explicit
  paths distinguish release tooling from its tests. All source is classified;
  macro-expansion and conditional-attribute diagnostics remain visible.
- All 12 root Cargo lint allowances and the adapter's broad nursery allowance
  were reviewed and removed. Actionable documentation/const/closure warnings
  were fixed. Scoped expectations preserve intentional exclusive mutation APIs,
  repository rustdoc spacing and private helper imports. Import expectations are
  conditional because Clippy suppresses that warning in test builds.
  `unfulfilled_lint_expectations` remains enabled; no source `allow` remains.
- Public fallible APIs now document their failure contracts. The helper borrows
  admitted release evidence instead of unnecessarily taking ownership.
- Captured native stderr is escaped for terminal presentation using Rust's
  standard escaping. Host's raw failure bytes and strict machine stdout decoding
  remain unchanged; a control-character/invalid-UTF-8 regression runs in the
  normal release-helper tests. Inherited interactive streams retain their contract.

## Other reviewed rules and retained choices

| Area | Verified application and boundary |
| --- | --- |
| Ownership and layout | Ordinary module discovery; no `#[path]` or executable source injection. Compiler/parser fixture source strings remain legitimate inputs. Tiny binary-owned host custody helpers retain separate instance ownership. |
| Dependencies | Root catalogs own dependency selections; Rust 1.99.1 remains pinned. Remaining registry/Git selections are preserved. The root lock drops the now-unused CDK-only closure; the independent lock adds the fixture and dependencies needed for the moved Logging canister. |
| Production graph | Three release groups remain unchanged. Native has no Platform runtime/types; Wasm backing uses Runtime's IC0 boundary. Testkit/PocketIC and ICSS comparison controls stay independent. |
| Errors and native effects | Existing `thiserror` owners remain. `PreparationError` retains computed rollback formatting and original-source selection. Host FS/Process own durable writes, bounded observation capture and process custody. Compilation/qualification has its deliberately longer lifecycle. |
| Serialization and observability | Owned messages use Wire. Logging/Metrics adapters remain governed persistence integrations, not replacement logging/statistics engines. No stored codec/layout change. |
| Tests | Existing `tests.rs`/`tests/` discovery is retained; no singular `test.rs`/`test/` owners. Source snippets in release fixtures qualify compilers/parsers rather than patch application implementations. |
| Validation | Actual consumer dispatch and archive integration are checked. Generic Shared regression suites remain with Shared. Optional final-candidate stage reuse is not enabled without its separate qualification. |
| Release and recovery | No package versions, tags, commits, pushes or registry publications. Failed evidence, historical archives, caches and existing preservation stashes remain. |
| Source and portability | No tracked symlink source owners. No new platform-specific syscall layer. Linux execution does not establish native macOS execution. |

## Evidence

Retained commands and logs: `target/shared-tooling-0300-adoption/`.

- Snapshot integrity, dependency/pin and production-graph admission.
- Native strict all-target Clippy, explicit `test-support` lint, Wasm all-target
  lint and moved fixture Wasm lint.
- 138 library tests, the existing compile-fail and example tests, and 88 helper
  tests including the new diagnostic rejection/presentation case.
- 22 independent unit tests (two installed tests remain ignored in unit runs),
  plus the moved allocation-history regression. The Logging native entrypoint is
  a zero-test stub; its behavioral proof comes from installed execution.
- Consumer validation/Make release adapter substitutes; these are not live releases.
- Exact archives for all three publishable packages qualify on native and Wasm
  under `target/package-qualification/attempt.P4c3BT/`.
- Installed original-Wasm qualification passes IO rejection, collection upgrades
  across 1,031 allocations, six directory failure checkpoints, backing custody
  and Logging rollback/reopen. See `installed.log`; no optimizer or macOS matrix
  was rerun for this layout change.
- LOC JSON reports cover both workspaces, with zero unclassified source.
- Formatting, maintained documentation links, changed-shell ShellCheck and
  Storage/ICSS benchmark Wasm lint pass.

Initial integration attempts exposed release-catalog/bootstrap assumptions about
private path dependencies. Those attempts failed and their logs are retained.
The final layout confines all fixture consumers to the independent workspace;
production release policy was not widened to accommodate test packages.
