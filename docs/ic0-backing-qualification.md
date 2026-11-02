# Storage 0.65.2 IC0-only backing qualification

Published on 2026-11-02 to crates.io, unyanked. Only the `storage` release group
was released and published. Metrics Storage remains 0.66.0 and Logging Storage
remains 0.63.0; Relay and sibling checkouts were not edited.

| Identity | Value |
| --- | --- |
| Package | `ic-storage` 0.65.2 |
| Implementation commit | `4db4f2ce5a18dcb6eeaaf0d8a75a21fdcd7d089d` |
| Release/source commit | `eac8a84c6d5cf860f906b3fb788d8f275a1745b5` |
| Annotated tag | [`storage-v0.65.2`](https://github.com/dragginzgame/ic-storage/tree/storage-v0.65.2) |
| Registry archive SHA-256 | `127f3898bc06ed65478490f2f052ee5455c432b76ca286de95bcaaa46195fa0a` |
| Compiler / host | Cargo and rustc 1.99.1 / `x86_64-unknown-linux-gnu` |

The [downloaded registry archive](https://static.crates.io/crates/ic-storage/ic-storage-0.65.2.crate),
fresh sparse-index checksum and final release receipt agree. Its
`.cargo_vcs_info.json` identifies the release commit and `crates/ic-storage` path.
Its normalized manifest requires `ic-platform-runtime` ^0.18.1, disables default
features and selects only `ic0` under `cfg(target_arch = "wasm32")`.
The downloaded backing source contains all four public safe `stable64_*` calls,
the `u64::MAX` refusal branch and checked successful page-count conversion.

## Qualification

- `make check-dependency-adoption` independently admitted and compiled the default
  external consumer and explicit Platform/runtime composition on native and Wasm.
  `make check-production-graph` passed the production normal/build exclusions.
  The caller's rejection fixtures passed, including unwanted default-native Runtime
  and default-Wasm executor activation.
- Both native consumer tests passed. The normal release runner's complete
  `make validate` passed in 230.067 seconds, including 78 release-adapter tests,
  85 Storage library tests, public runtime tests, compiler admission, strict lint,
  documentation, formatting, Wasm checks and whole-workspace archive integration.
- `make test-runtime` passed original, O3, Os and Oz Wasm against the admitted
  Testkit 0.79.0 / PocketIC 16.1.0 server. Each variant covered valid IO, growth
  success/refusal, 12 invalid-IO traps, neighboring-data preservation, reopen,
  upgrade and rollback, six directory fault checkpoints, exclusive backing
  custody, all five collection forms and Logging checkpoints. The installed
  candidate source is unchanged by release version preparation.
- After publication, a fresh external consumer selected registry `ic-storage`
  exactly 0.65.2 with default features. Native `cargo run` and Wasm `cargo build
  --lib` passed independently. The same commands passed with explicit optional
  Platform/runtime composition. All builds used the generated lockfile offline;
  no Git pins, dependency paths, registry patches or guard overrides were used.
  Metadata for default and composed selections contains only registry dependencies.

| Registry consumer | Runtime selection | Execution guard |
| --- | --- | --- |
| Default native | Absent | Absent |
| Default Wasm | 0.18.1, exactly `ic0` | Absent |
| Runtime composition, native and Wasm | One 0.18.1, `ic0`, `executor`, `runtime` | One 0.18.1 |

Default Wasm also excludes CDK, Types and Candid. The fresh registry selection
resolved Wire/derive 0.15.1; explicit composition selected Platform 0.18.0 and
Types 0.17.0. These are compatible registry selections, not new consumer minimums.
Retained formats, bounds traps and explicit host reset/migration authority remain
unchanged. No compatibility reader or automatic reset was introduced.

## Evidence custody

The isolated release checkout is `/tmp/ic-storage-ic0-release-0652`. Its retained
evidence paths are:

- `target/storage-0652-deploy/`: tool/compiler and graph checks, consumer tests,
  release/publication logs, downloaded archive/index, verification script/results,
  four registry graph observations and native/Wasm build logs.
- `target/release-validation/attempts/verify.S0HqOf`: complete successful gate.
- `target/release-validation/storage-0.65.2.json`: final source-bound archive receipt.
- `target/package-qualification/attempt.QHiehT/`: final archive integration proof.
- `target/qualification/runtime/attempt.VzJL5A/`: installed optimizer matrix,
  artifact/input digests and individual runtime logs.

The initial sandbox loopback refusal remains in runtime attempt `WmGAXB`; the
preflight DNS refusal and Node subprocess-observation refusal remain in the
deployment logs. Subsequent authorized attempts passed. These are Linux-local
results; they do not claim native macOS execution. Concurrent benchmark work in
the original checkout was preserved and excluded from this release.
