# Persistence release notes

Covers `ic-memory` and `ic-storage`. Independent group releases begin after the
coordinated 0.58.1 cutoff; [earlier shared history](../history.md) is preserved once.

## [0.61.1] - Unreleased

### 🔧 Changed

- Offer exact-input reuse for the real ShellCheck tooling stage with fresh group
  metadata and source/index admission. Retain the original execution proof and
  separate current admissions; existing complete release gates remain unchanged.
  [Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334).

- Use Shared's source-only grouped Cargo observer for interrupted launcher
  startup. Recheck metadata after tool/cache helpers and verify copied inputs
  before scratch rendering, preserving native source/receipt authorization.
  [#333 — Review a shared read-only observation boundary for grouped Cargo bootstrap](https://github.com/dragginzgame/shared-tooling/issues/333).

## [0.61.0] - 2026-11-01

### ⚠️ Breaking

- `LedgerCommitError::Codec` now carries `LedgerDecodeError` with typed Wire,
  stable-key and slot sources. Callers constructing or inspecting that payload
  must update; existing durable bytes and recovery admission remain unchanged.

- Owning Storage collections now use fresh-only `create`, retained-only `load`,
  or deliberate `reset`; combined initialization is removed. `StableCell` always
  enforces declared record bounds and replaces the separate bounded wrapper.
- Runtime lookup/open takes `&StableKey`, validated once at setup. Bucket
  configuration takes `NonZeroU16` and is infallible. Stored bytes are unchanged;
  callers must migrate their constructors and key/config setup. [Migration](../../docs/migration/0.61-public-api.md).

### 🧹 Cleanup

- Delegate archive extraction, graph admission and input custody to Build 0.73.5,
  retaining the consumer's library/native/MSRV/Wasm roster. Carry inspected release
  evidence within a phase, classify metadata roles before rendering, and simplify
  fixture metadata and temporary-name allocation. [Details](0.61.md).

## [0.60.7] - 2026-11-01

### ⬆️ Dependencies

- Adopt Wire 0.13.3 in both maintained catalogs and Testkit Native/CLI 0.76.0.
  Native Testkit now shares Platform 0.16.0; the PocketIC 16.1.0 pairing,
  existing encoded bytes and stable-memory contracts remain unchanged. [Details](0.60.md).

## [0.60.6] - 2026-11-01

### 🔧 Changed

- Bound native receipt reads and centralize typed JSON admission through Host FS.
  Give captured Git/Cargo/helper observations a 120-second deadline while leaving
  inherited-IO builds and gates unchanged. Adopt Platform 0.16.0 in both catalogs,
  preserving stable-structures 0.7.2, PocketIC 16.1.0 and stored formats.

## [0.60.5] - 2026-11-01

### 🧹 Cleanup

- Simplify release evidence flow, phase-local group projection, command plans
  and interrupted-bootstrap observation. Separate pure growth calculations from
  mutation, derive diagnostic status from one recovery observation and initialize
  release fixture groups together. Public contracts and receipt bytes are unchanged.
  [Details](0.60.md).

## [0.60.4] - 2026-11-01

### ⬆️ Dependencies

- Adopt Wire 0.13.2 in both maintained graphs, Build/Host/editor 0.73.3 and
  Testkit Native/CLI 0.75.2. Preserve codec bytes, stable-memory helpers and the
  PocketIC 16.1.0 pairing; no retained-data migration is needed.
  [Details](0.60.md).

## [0.60.3] - 2026-10-31

### 🩹 Fixed

- Fail every required native matrix check explicitly when classification or
  documentation admission fails. Preserve full-suite dispatch and docs-only
  reporting without relying on skipped dependency checks for refusal
  ([#145](https://github.com/dragginzgame/ic-persistence/issues/145)).

## [0.60.2] - 2026-10-31

### 🔧 Changed

- Add conservative documentation-only CI selection for three reviewed prose
  paths, with real documentation/snapshot checks and stable required check names.
  Code, policy, ambiguous history and failures retain full checks
  ([#145](https://github.com/dragginzgame/ic-persistence/issues/145)).

### ⬆️ Dependencies

- Adopt Platform 0.15.3 and Build/Host/editor 0.73.2. Preserve Wire 0.13,
  Testkit 0.75, stable-structures 0.7.2 and PocketIC 16.1.0.

### 🩹 Fixed

- Invoke the selected Rust toolchain explicitly in isolated hook fixtures,
  avoiding reliance on Rustup proxy-name dispatch. Local qualification passes;
  macOS acceptance remains pending
  ([#146](https://github.com/dragginzgame/ic-persistence/issues/146)).

## [0.60.1] - 2026-10-31

### 🔧 Changed

- Adopt Shared Tooling’s 0.27.22 rules (commit `143a9ab`, recorded VERSION
  0.27.21). Release preparation keeps the root’s versioned
  local dependencies aligned with their current packages, including patch
  releases, while preserving requirement operators and independent adapter versions.
  The root catalog includes every member and its owning release-policy alias;
  unused declarations add no dependency edges.

## [0.60.0] - 2026-10-31

### ⚠️ Breaking

- Adopt IC Wire 0.13.0 for durable codecs. Consumers exchanging exposed Wire
  errors must align that dependency identity. Encoded bytes, bounds and admission
  remain unchanged; no retained-data reset or migration is required.

### ⬆️ Dependencies

- Adopt Build/Host 0.73.1 and Testkit Native/CLI 0.75.0. Preserve the qualified
  stable-memory substrate and PocketIC 16.1.0 server selection.

## [0.59.0] - 2026-10-31

### ⚠️ Breaking

- Adopt IC Wire 0.12.1 in both maintained workspaces. Consumers exchanging
  Memory codec errors must align that dependency identity. Stored bytes, bounds and codec admission are unchanged.
  [Details](0.59.md#0590---unreleased).

### ⬆️ Dependencies

- Adopt Platform 0.15.1 in both maintained workspaces and Testkit 0.73.0.
  Preserve storage, runtime custody and the qualified IC substrates.

### 🔧 Changed

- Release Memory/Storage together and each stable adapter independently, retaining
  one workspace and existing numeric versions ([#144 — Separate core persistence and stable adapters into independent release groups](https://github.com/dragginzgame/ic-persistence/issues/144)).


- Adopt Shared Tooling 0.27.19, including symmetric group ledgers and explicit
  delegated delivery while preserving release and publication checks.
