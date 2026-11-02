# Current handoff

Delivery selected: Persistence 0.63.0, Metrics Stable 0.65.0 and Logging Stable
0.63.0 to crates.io and GitHub `dragginzgame/ic-persistence`, `main`, followed by
closure of #151 after verified publication. Standard group gates and final
registry/tag readback own delivery evidence under `target/delivery/rust-1.99.1-2026-11-02/`.
Preparation-only wording below predates this explicit delivery selection.

Current work implements [#151](https://github.com/dragginzgame/ic-persistence/issues/151):
the maintainer selected Rust 1.99.1 as both support floor and pinned compiler.
Root and independent runtime catalogs now declare 1.99.1; all five packages
inherit their owning catalog and the pin already selected 1.99.1. Local instructions
record the explicit exception to the reviewed baseline's earlier floor guidance;
no vendored policy or sibling checkout changed.

Compiler admission checks both catalogs, package inheritance and the pin before
probing actual Cargo/rustc identities once each. The release helper refuses
mismatched policy or observed versions and preserves its receipt roles without
repeating the same rustc observation. Extracted libraries still compile separately;
the feature-enabled native archive consumer runs once, followed by Wasm.

The duplicate MSRV CI matrix and second compiler installation are removed. Existing
native and installed-runtime jobs retain Linux and both macOS hosts. The old
MSRV job's distinct native `--all-targets --features test-support` proof is now
in `validate-toolchain`, after Clippy. Default compilation, proc macros, Wasm test
compilation, raw size budgets, archive integration and installed-runtime/optimizer
obligations remain. `validate` delegates once rather than replaying a default graph.
Classic branch protection returned unprotected and active branch rules were empty;
no required-check reference needed remote modification. CI fixtures retain
classification/docs failure propagation and both three-host matrices.

Support-floor changes select pending minor notes: Persistence 0.63.0, Metrics
Stable 0.65.0 and Logging Stable 0.63.0. Hosts using older Rust must upgrade.
Public runtime APIs, stable formats and custody are unchanged. Package versions,
staging, commits, publication and issue closure are not selected by this fix.

Focused evidence is retained under `target/issue-151/`. Compiler admission/refusal
fixtures, release Make adapters, workflow selection, selected ShellCheck and
Actionlint, helper Clippy, configuration tests and note-finalization tests passed.
All 77 release-helper tests passed across the focused selections and remaining
regression run. Production graph guards and release-note links also passed.
Native all-target `test-support`, Wasm workspace test graphs and the independent
qualifier compiled on 1.99.1 with locked dependencies. Real archive qualification
passed under `target/package-qualification/attempt.jYwzvw/`: four library checks,
one native consumer and one Wasm consumer, with final source/archive custody.
The archive fixture confirms one native dispatch and early substituted-payload
refusal. Full release gates and installed/native macOS execution were not replayed.
Earlier failed subset ShellCheck attempts are retained; the proper source
companions were included in the passing invocation.

Incoming `Cargo.lock` edits (crc32fast, Metrics and Wire patches) remain unchanged;
the original lock and patch are retained in `target/issue-151/`. The independent
lockfile and all tool/server selections are unchanged. No cache or artifact was
removed. Existing installed qualification remains scoped to its recorded inputs.

Latest publication is Persistence 0.62.1 (Memory/Storage), Metrics Stable 0.64.0
and Logging Stable 0.62.1, with all group gates, final checksums, requirements and
tags verified. Evidence remains under
`target/delivery/metrics-testkit-wire-2026-11-01/verification.json`. The retained
failed gate `verify.7uETZU` was repaired by seeding declared group metadata in
fixtures; its source-bound retry and later delivery passed. The first failed
Persistence 0.61.0 plan is superseded evidence and must not be resumed.

Earlier Wire/Platform and public API hard-cut delivery/qualification evidence
remains in its original `target/delivery/`, `target/wire-0.14-adoption/`,
`target/platform-0.17-adoption/` and `target/hard-cut-api-2026-11-01/` directories.
See [the public API migration](../migration/0.61-public-api.md) for constructor/key
changes. Stored formats remain unchanged by the compiler decision.

Build owns extracted archive admission/input custody; the consumer owns its
qualification roster. Shared's grouped observer is adopted from
[PR #336](https://github.com/dragginzgame/shared-tooling/pull/336). The merged
native destination check covers [Shared #335](https://github.com/dragginzgame/shared-tooling/issues/335).
Opt-in ShellCheck proof reuse and native acceptance remain tracked by
[Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334).
The reviewed snapshot remains `9275bf0de39ed941d81c47bab97eef7f6fc38df2`
(VERSION 0.27.24), with 81 roots and 136 payloads. Never patch vendored files.
